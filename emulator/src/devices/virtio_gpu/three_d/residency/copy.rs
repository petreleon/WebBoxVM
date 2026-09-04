use super::ResidentResource;
use crate::devices::virtio_gpu::VirtioGpu;
use crate::devices::virtio_gpu::protocol::{CtrlHeader, Rect, RESP_ERR_INVALID_PARAMETER, RESP_ERR_OUT_OF_MEMORY};
use crate::devices::virtio_gpu::three_d::virgl::CopyRegion;
use crate::devices::virtio_gpu::three_d::{BrowserCompletion, DeferredSubmit, Pending3d, Pending3dEffect};
use crate::devices::virtio_gpu::{MAX_PENDING_3D_BYTES, MAX_PENDING_3D_SUBMITS};

impl VirtioGpu {
    pub(in crate::devices::virtio_gpu) fn resident_copy_in_flight(&self, resource_id: u32) -> bool {
        self.pending_3d.iter().any(|pending| matches!(pending.effect.as_ref(),
            Some(Pending3dEffect::VirglResidentCopy { resource_id: target, source_resource_id, .. })
                if *target == resource_id || *source_resource_id == resource_id))
    }

    pub(in crate::devices::virtio_gpu) fn resident_copy_eligible(
        &self, context_id: u32, generation: u32, copy: CopyRegion, destination: Rect,
    ) -> bool {
        let Some(source) = self.resources.get(&copy.src_resource) else { return false; };
        let Some(target) = self.resources.get(&copy.dst_resource) else { return false; };
        let Some(resident) = self.resident_resources.get(&copy.src_resource) else { return false; };
        copy.src_resource != copy.dst_resource
            && !self.resident_copy_in_flight(copy.src_resource)
            && !self.resident_copy_in_flight(copy.dst_resource)
            && resident.context_id == context_id && resident.generation == generation
            && !self.resident_resources.contains_key(&copy.dst_resource)
            && source.is_texture_2d() && target.is_texture_2d()
            && copy.src_rect == full_rect(source.width, source.height)
            && destination == full_rect(target.width, target.height)
            && source.width == target.width && source.height == target.height
            && self.resident_target_eligible(copy.dst_resource, destination)
    }

    pub(in crate::devices::virtio_gpu) fn queue_resident_copy(
        &mut self, header: CtrlHeader, generation: u32, copy: CopyRegion,
    ) -> Result<DeferredSubmit, u32> {
        let target = self.resources.get(&copy.dst_resource).ok_or(RESP_ERR_INVALID_PARAMETER)?;
        let rect = full_rect(target.width, target.height);
        if !self.resident_copy_eligible(header.ctx_id, generation, copy, rect) {
            return Err(RESP_ERR_INVALID_PARAMETER);
        }
        if self.pending_3d.len() >= MAX_PENDING_3D_SUBMITS { return Err(RESP_ERR_OUT_OF_MEMORY); }
        let source = *self.resident_resources.get(&copy.src_resource).expect("resident copy validated");
        let mut sequence = self.allocate_3d_sequence().ok_or(RESP_ERR_OUT_OF_MEMORY)?;
        if sequence == source.producer_sequence {
            sequence = self.allocate_3d_sequence().ok_or(RESP_ERR_OUT_OF_MEMORY)?;
        }
        let packet = copy_packet(sequence, source.producer_sequence, rect);
        if self.pending_3d_bytes.checked_add(packet.len()).is_none_or(|total| total > MAX_PENDING_3D_BYTES) {
            return Err(RESP_ERR_OUT_OF_MEMORY);
        }
        self.pending_3d_bytes += packet.len();
        self.pending_3d.push(Pending3d {
            sequence, timeline: self.fence_timeline(header), bytes: packet.len(), packet: Some(packet), completion: None,
            effect: Some(Pending3dEffect::VirglResidentCopy {
                context_id: header.ctx_id, generation, resource_id: copy.dst_resource,
                source_resource_id: copy.src_resource, source_producer_sequence: source.producer_sequence,
                rect, resident_epoch: self.resident_epoch,
            }),
            browser_completion: BrowserCompletion::Resident,
        });
        Ok(DeferredSubmit { sequence, header })
    }

    pub(in crate::devices::virtio_gpu) fn resident_copy_effect_valid(&self, effect: &Pending3dEffect) -> bool {
        let Pending3dEffect::VirglResidentCopy {
            context_id, generation, resource_id, source_resource_id, source_producer_sequence, rect, resident_epoch,
        } = effect else { return false; };
        let source = self.resources.get(source_resource_id);
        let target = self.resources.get(resource_id);
        let owner = ResidentResource {
            context_id: *context_id, generation: *generation, producer_sequence: *source_producer_sequence,
        };
        self.resident_epoch == *resident_epoch
            && self.virgl_contexts.get(context_id).is_some_and(|context|
                context.generation == *generation && context.is_attached(*source_resource_id) && context.is_attached(*resource_id))
            && self.resident_resources.get(source_resource_id) == Some(&owner)
            && !self.resident_resources.contains_key(resource_id)
            && source.is_some_and(|resource| resource.is_texture_2d() && full_rect(resource.width, resource.height) == *rect)
            && target.is_some_and(|resource| resource.is_texture_2d() && full_rect(resource.width, resource.height) == *rect)
            && self.resident_target_eligible(*resource_id, *rect)
            && !self.scanout.is_some_and(|scanout| scanout.resource_id == *source_resource_id || scanout.resource_id == *resource_id)
    }
}

fn full_rect(width: u32, height: u32) -> Rect { Rect { x: 0, y: 0, width, height } }

fn copy_packet(sequence: u32, producer: u32, rect: Rect) -> Vec<u8> {
    let mut packet = b"VRC1".to_vec();
    for value in [1, sequence, producer, rect.width, rect.height] { packet.extend_from_slice(&value.to_le_bytes()); }
    packet
}
