use super::{BrowserCompletion, DeferredSubmit, Pending3d, Pending3dEffect};
use crate::devices::virtio_gpu::VirtioGpu;
use crate::devices::virtio_gpu::protocol::{CtrlHeader, RESP_ERR_INVALID_PARAMETER, RESP_ERR_OUT_OF_MEMORY, RESP_ERR_UNSPEC, Rect};
use crate::devices::virtio_gpu::{MAX_PENDING_3D_BYTES, MAX_PENDING_3D_SUBMITS};
use crate::memory::PhysicalMemory;

const MAX_RESIDENT_BYTES: usize = 4 * 1024 * 1024;
const MAX_RESIDENT_RESOURCES: usize = 4;
const MAX_SNAPSHOT_DIMENSION: u32 = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::devices::virtio_gpu) struct ResidentResource {
    pub context_id: u32,
    pub generation: u32,
    pub producer_sequence: u32,
}

impl VirtioGpu {
    pub(in crate::devices::virtio_gpu) fn resident_candidate(
        &self,
        packet: &[u8],
        effect: &Pending3dEffect,
    ) -> bool {
        packet.get(..4) == Some(b"VGB1")
            && matches!(effect, Pending3dEffect::VirglBatch { .. })
            && self.resident_effect_valid(effect)
    }

    pub(in crate::devices::virtio_gpu) fn resident_target_eligible(
        &self,
        resource_id: u32,
        rect: crate::devices::virtio_gpu::protocol::Rect,
    ) -> bool {
        let Some(resource) = self.resources.get(&resource_id) else { return false; };
        let full = rect.x == 0 && rect.y == 0 && rect.width == resource.width && rect.height == resource.height;
        let room = !self.resident_resources.contains_key(&resource_id)
            && self.resident_resources.len() < MAX_RESIDENT_RESOURCES;
        full && room && resource.is_texture_2d() && resource.pixels.len() <= MAX_RESIDENT_BYTES
            && (resource.width > MAX_SNAPSHOT_DIMENSION || resource.height > MAX_SNAPSHOT_DIMENSION)
    }

    pub(in crate::devices::virtio_gpu) fn resident_overwrite_allowed(
        &self,
        resource_id: u32,
        rect: Rect,
    ) -> bool {
        !self.resident_resources.contains_key(&resource_id) || self.resources.get(&resource_id)
            .is_some_and(|resource| rect.x == 0 && rect.y == 0 && rect.width == resource.width && rect.height == resource.height)
    }

    pub(in crate::devices::virtio_gpu) fn promote_resident(
        &mut self,
        sequence: u32,
        effect: Pending3dEffect,
    ) -> bool {
        if !self.resident_effect_valid(&effect) { return false; }
        let Pending3dEffect::VirglBatch { context_id, generation, resource_id, .. } = effect else { return false; };
        self.resident_resources.insert(resource_id, ResidentResource {
            context_id, generation, producer_sequence: sequence,
        });
        if self.scanout.is_some_and(|scanout| scanout.resource_id == resource_id) {
            self.pending_damage = None;
        }
        true
    }

    pub(in crate::devices::virtio_gpu) fn forget_resident(&mut self, resource_id: u32) {
        self.resident_resources.remove(&resource_id);
    }

    pub(in crate::devices::virtio_gpu) fn queue_resident_readback(
        &mut self,
        header: CtrlHeader,
        resource_id: u32,
        transfer_rect: Rect,
        transfer_offset: u64,
    ) -> Result<DeferredSubmit, u32> {
        let resident = *self.resident_resources.get(&resource_id).ok_or(RESP_ERR_INVALID_PARAMETER)?;
        if self.pending_3d.iter().any(|pending| matches!(pending.effect.as_ref(),
            Some(Pending3dEffect::VirglResidentReadback { resource_id: pending_id, .. }) if *pending_id == resource_id)) {
            return Err(RESP_ERR_UNSPEC);
        }
        let resource = self.resources.get(&resource_id).ok_or(RESP_ERR_INVALID_PARAMETER)?;
        let source_rect = Rect { x: 0, y: 0, width: resource.width, height: resource.height };
        if self.pending_3d.len() >= MAX_PENDING_3D_SUBMITS {
            return Err(RESP_ERR_OUT_OF_MEMORY);
        }
        if !self.resident_context_valid(resident) { return Err(RESP_ERR_INVALID_PARAMETER); }
        let sequence = self.allocate_3d_sequence().ok_or(RESP_ERR_OUT_OF_MEMORY)?;
        let packet = readback_packet(sequence, resident.producer_sequence, source_rect.width, source_rect.height);
        if self.pending_3d_bytes.checked_add(packet.len()).is_none_or(|total| total > MAX_PENDING_3D_BYTES) {
            return Err(RESP_ERR_OUT_OF_MEMORY);
        }
        self.pending_3d_bytes += packet.len();
        self.pending_3d.push(Pending3d {
            sequence, timeline: self.fence_timeline(header), bytes: packet.len(), packet: Some(packet), completion: None,
            effect: Some(Pending3dEffect::VirglResidentReadback {
                context_id: resident.context_id, generation: resident.generation, resource_id,
                producer_sequence: resident.producer_sequence, source_rect, transfer_rect, transfer_offset,
            }),
            browser_completion: BrowserCompletion::Readback,
        });
        Ok(DeferredSubmit { sequence, header })
    }

    pub(in crate::devices::virtio_gpu) fn resolve_resident_readback(
        &mut self,
        mem: &mut PhysicalMemory,
        effect: Pending3dEffect,
        format: u32,
        pixels: &[u8],
    ) -> bool {
        let Pending3dEffect::VirglResidentReadback {
            context_id, generation, resource_id, producer_sequence, source_rect, transfer_rect, transfer_offset,
        } = effect else { return false; };
        let resident = self.resident_resources.get(&resource_id).copied();
        if resident != Some(ResidentResource { context_id, generation, producer_sequence })
            || !self.write_gpu_readback(resource_id, source_rect, format, pixels) {
            return false;
        }
        self.forget_resident(resource_id);
        self.resources.get(&resource_id).is_some_and(|resource|
            resource.transfer_from_host(mem, transfer_rect, transfer_offset).is_some())
    }

    fn resident_effect_valid(&self, effect: &Pending3dEffect) -> bool {
        let Pending3dEffect::VirglBatch { context_id, generation, resource_id, rect, .. } = effect else {
            return false;
        };
        let context_valid = self.virgl_contexts.get(context_id)
            .is_some_and(|context| context.generation == *generation);
        context_valid && self.resident_target_eligible(*resource_id, *rect)
    }

    fn resident_context_valid(&self, resident: ResidentResource) -> bool {
        self.virgl_contexts.get(&resident.context_id)
            .is_some_and(|context| context.generation == resident.generation)
    }
}

fn readback_packet(sequence: u32, producer: u32, width: u32, height: u32) -> Vec<u8> {
    let mut packet = b"VGR1".to_vec();
    for value in [1, sequence, producer, width, height] { packet.extend_from_slice(&value.to_le_bytes()); }
    packet
}
