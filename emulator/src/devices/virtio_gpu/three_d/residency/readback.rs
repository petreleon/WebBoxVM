use super::ResidentResource;
use crate::devices::virtio_gpu::VirtioGpu;
use crate::devices::virtio_gpu::protocol::{CtrlHeader, Rect, RESP_ERR_INVALID_PARAMETER, RESP_ERR_OUT_OF_MEMORY, RESP_ERR_UNSPEC};
use crate::devices::virtio_gpu::three_d::{BrowserCompletion, DeferredSubmit, Pending3d, Pending3dEffect};
use crate::devices::virtio_gpu::{MAX_PENDING_3D_BYTES, MAX_PENDING_3D_SUBMITS};
use crate::memory::PhysicalMemory;

impl VirtioGpu {
    pub(in crate::devices::virtio_gpu) fn queue_resident_readback(
        &mut self, header: CtrlHeader, resource_id: u32, transfer_rect: Rect, transfer_offset: u64,
    ) -> Result<DeferredSubmit, u32> {
        let resident = *self.resident_resources.get(&resource_id).ok_or(RESP_ERR_INVALID_PARAMETER)?;
        if self.pending_3d.iter().any(|pending| matches!(pending.effect.as_ref(),
            Some(Pending3dEffect::VirglResidentReadback { resource_id: pending_id, .. }) if *pending_id == resource_id)) {
            return Err(RESP_ERR_UNSPEC);
        }
        let resource = self.resources.get(&resource_id).ok_or(RESP_ERR_INVALID_PARAMETER)?;
        let target_rect = full_rect(resource.width, resource.height);
        if !resource.is_texture_2d() || !transfer_rect.valid_within(resource.width, resource.height) {
            return Err(RESP_ERR_INVALID_PARAMETER);
        }
        if self.pending_3d.len() >= MAX_PENDING_3D_SUBMITS { return Err(RESP_ERR_OUT_OF_MEMORY); }
        if !self.resident_context_valid(resident) { return Err(RESP_ERR_INVALID_PARAMETER); }
        let sequence = self.allocate_3d_sequence().ok_or(RESP_ERR_OUT_OF_MEMORY)?;
        let source_rect = transfer_rect;
        let packet = readback_packet(sequence, resident.producer_sequence, target_rect, source_rect);
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
        &mut self, mem: &mut PhysicalMemory, effect: Pending3dEffect, format: u32, pixels: &[u8],
    ) -> bool {
        let Pending3dEffect::VirglResidentReadback {
            context_id, generation, resource_id, producer_sequence, source_rect, transfer_rect, transfer_offset,
        } = effect else { return false; };
        if self.resident_resources.get(&resource_id).copied()
            != Some(ResidentResource { context_id, generation, producer_sequence }) {
            return false;
        }
        let partial = self.resources.get(&resource_id)
            .is_some_and(|resource| source_rect != full_rect(resource.width, resource.height));
        if partial {
            return source_rect == transfer_rect && self.resources.get(&resource_id).is_some_and(|resource|
                resource.transfer_gpu_readback_from_host(mem, source_rect, transfer_offset, format, pixels).is_some());
        }
        if !self.write_gpu_readback(resource_id, source_rect, format, pixels) { return false; }
        self.advance_resident_epoch();
        self.resident_resources.remove(&resource_id);
        self.resources.get(&resource_id).is_some_and(|resource|
            resource.transfer_from_host(mem, transfer_rect, transfer_offset).is_some())
    }
}

fn full_rect(width: u32, height: u32) -> Rect { Rect { x: 0, y: 0, width, height } }

fn readback_packet(sequence: u32, producer: u32, target: Rect, source: Rect) -> Vec<u8> {
    let partial = source != target;
    let mut packet = b"VGR1".to_vec();
    for value in [if partial { 2 } else { 1 }, sequence, producer, target.width, target.height] {
        packet.extend_from_slice(&value.to_le_bytes());
    }
    if partial {
        for value in [source.x, source.y, source.width, source.height] {
            packet.extend_from_slice(&value.to_le_bytes());
        }
    }
    packet
}
