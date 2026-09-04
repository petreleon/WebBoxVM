use super::VirtioGpu;
use crate::constants::VIRTIO_GPU_HOST_VISIBLE_BASE;
use crate::memory::PhysicalMemory;

impl VirtioGpu {
    pub fn cold_reset(&mut self, mem: &mut PhysicalMemory) {
        self.discard_host_visible_mappings(mem);
        let next_3d_sequence = self.next_3d_sequence;
        let reset_generation = self.reset_generation.wrapping_add(1);
        *self = Self::new();
        self.next_3d_sequence = next_3d_sequence;
        self.reset_generation = reset_generation;
    }

    fn discard_host_visible_mappings(&self, mem: &mut PhysicalMemory) {
        for blob in self.blobs.values() {
            let Some((offset, size)) = blob.mapped_range() else {
                continue;
            };
            if let Some(address) = VIRTIO_GPU_HOST_VISIBLE_BASE.checked_add(offset) {
                let _ = mem.discard_range(address, size);
            }
        }
    }
}
