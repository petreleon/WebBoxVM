use super::super::MAX_TOTAL_RESOURCE_BYTES;
use super::super::protocol::{
    CTRL_HEADER_LEN, RESP_ERR_INVALID_PARAMETER, RESP_ERR_INVALID_RESOURCE_ID, RESP_ERR_UNSPEC,
    RESP_OK_NODATA, read_u32,
};
use crate::constants::VIRTIO_GPU_HOST_VISIBLE_BASE;
use crate::memory::PhysicalMemory;

impl super::super::VirtioGpu {
    pub(in crate::devices::virtio_gpu) fn unref_resource(
        &mut self,
        mem: &mut PhysicalMemory,
        input: &[u8],
    ) -> u32 {
        if input.len() < 32 {
            return RESP_ERR_INVALID_PARAMETER;
        }
        let Some(resource_id) = read_u32(input, CTRL_HEADER_LEN) else {
            return RESP_ERR_INVALID_PARAMETER;
        };
        let mapped = self
            .blobs
            .get(&resource_id)
            .and_then(|blob| blob.mapped_range());
        if let Some((offset, size)) = mapped {
            let Some(address) = VIRTIO_GPU_HOST_VISIBLE_BASE.checked_add(offset) else {
                return RESP_ERR_UNSPEC;
            };
            if mem.discard_range(address, size).is_none() {
                return RESP_ERR_UNSPEC;
            }
        }
        let bytes = if let Some(resource) = self.resources.remove(&resource_id) {
            resource.pixels.len()
        } else if let Some(mut blob) = self.blobs.remove(&resource_id) {
            let bytes = blob.size;
            blob.backing.clear();
            bytes
        } else {
            return RESP_ERR_INVALID_RESOURCE_ID;
        };
        self.allocated_resource_bytes = self.allocated_resource_bytes.saturating_sub(bytes);
        self.remove_virgl_resource(resource_id);
        self.detach_scanout_resource(resource_id);
        RESP_OK_NODATA
    }
}

pub(in crate::devices::virtio_gpu) fn total_resource_limit(current: usize, added: usize) -> bool {
    current
        .checked_add(added)
        .is_some_and(|total| total <= MAX_TOTAL_RESOURCE_BYTES)
}
