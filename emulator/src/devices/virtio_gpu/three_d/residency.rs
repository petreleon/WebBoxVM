use crate::devices::virtio_gpu::VirtioGpu;
use crate::devices::virtio_gpu::protocol::Rect;

const MAX_RESIDENT_BYTES: usize = 4 * 1024 * 1024;
const MAX_RESIDENT_RESOURCES: usize = 16;
const MAX_RESIDENT_TOTAL_BYTES: usize = 16 * 1024 * 1024;
const MAX_RESIDENT_RELEASES: usize = 16;
const MAX_SNAPSHOT_DIMENSION: u32 = 64;

mod promotion;
mod readback;
mod copy;
mod sample;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::devices::virtio_gpu) struct ResidentResource {
    pub context_id: u32,
    pub generation: u32,
    pub producer_sequence: u32,
}

impl VirtioGpu {
    pub(in crate::devices::virtio_gpu) fn resident_target_eligible(
        &self,
        resource_id: u32,
        rect: crate::devices::virtio_gpu::protocol::Rect,
    ) -> bool {
        let Some(resource) = self.resources.get(&resource_id) else { return false; };
        let full = rect.x == 0 && rect.y == 0 && rect.width == resource.width && rect.height == resource.height;
        let existing = self.resident_resources.contains_key(&resource_id);
        let room = existing || (self.resident_resources.len() < MAX_RESIDENT_RESOURCES
            && self.resident_bytes().and_then(|total| total.checked_add(resource.pixels.len()))
                .is_some_and(|total| total <= MAX_RESIDENT_TOTAL_BYTES));
        full && room && resource.is_texture_2d() && resource.pixels.len() <= MAX_RESIDENT_BYTES
            && (resource.width > MAX_SNAPSHOT_DIMENSION || resource.height > MAX_SNAPSHOT_DIMENSION)
    }

    fn resident_bytes(&self) -> Option<usize> {
        self.resident_resources.keys().try_fold(0usize, |total, resource_id| {
            self.resources.get(resource_id)?.pixels.len().checked_add(total)
        })
    }

    pub(in crate::devices::virtio_gpu) fn resident_overwrite_allowed(
        &self,
        resource_id: u32,
        rect: Rect,
    ) -> bool {
        !self.resident_resource_in_flight(resource_id)
            && (!self.resident_resources.contains_key(&resource_id) || self.resources.get(&resource_id)
                .is_some_and(|resource| rect.x == 0 && rect.y == 0 && rect.width == resource.width && rect.height == resource.height))
    }

    pub(in crate::devices::virtio_gpu) fn resident_resource_in_flight(&self, resource_id: u32) -> bool {
        self.resident_copy_in_flight(resource_id) || self.resident_sample_in_flight(resource_id)
    }

    pub(in crate::devices::virtio_gpu) fn forget_resident(&mut self, resource_id: u32) {
        self.advance_resident_epoch();
        let Some(resident) = self.resident_resources.remove(&resource_id) else { return; };
        self.queue_resident_release(resident.producer_sequence);
    }

    pub(in crate::devices::virtio_gpu) fn queue_resident_release(&mut self, producer_sequence: u32) {
        if self.resident_releases.len() < MAX_RESIDENT_RELEASES {
            self.resident_releases.push_back(producer_sequence);
        }
    }

    pub(in crate::devices::virtio_gpu) fn forget_resident_context(&mut self, context_id: u32) {
        let resources: Vec<u32> = self.resident_resources.iter().filter_map(|(&resource_id, resident)|
            (resident.context_id == context_id).then_some(resource_id)).collect();
        for resource_id in resources { self.forget_resident(resource_id); }
    }

    fn resident_context_valid(&self, resident: ResidentResource) -> bool {
        self.virgl_contexts.get(&resident.context_id)
            .is_some_and(|context| context.generation == resident.generation)
    }

    fn advance_resident_epoch(&mut self) {
        self.resident_epoch = self.resident_epoch.wrapping_add(1);
    }
}

pub(in crate::devices::virtio_gpu) fn release_packet(producer: u32) -> Vec<u8> {
    let mut packet = b"VGL1".to_vec();
    for value in [1, producer] { packet.extend_from_slice(&value.to_le_bytes()); }
    packet
}
