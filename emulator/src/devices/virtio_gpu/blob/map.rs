use super::super::protocol::*;
use super::super::{VIRTIO_GPU_F_RESOURCE_BLOB, VirtioGpu};
use crate::constants::{PAGE_SIZE, VIRTIO_GPU_HOST_VISIBLE_BASE, VIRTIO_GPU_HOST_VISIBLE_SIZE};
use crate::memory::PhysicalMemory;

const MAP_CACHE_CACHED: u32 = 1;

impl VirtioGpu {
    pub(in crate::devices::virtio_gpu) fn map_blob(
        &mut self,
        mem: &mut PhysicalMemory,
        header: CtrlHeader,
        input: &[u8],
    ) -> Vec<u8> {
        let response = match self.map_blob_inner(mem, input) {
            Ok(()) => RESP_OK_MAP_INFO,
            Err(error) => return header.encode(error),
        };
        let mut out = header.encode(response);
        push_u32(&mut out, MAP_CACHE_CACHED);
        push_u32(&mut out, 0);
        out
    }

    pub(in crate::devices::virtio_gpu) fn unmap_blob(
        &mut self,
        mem: &mut PhysicalMemory,
        input: &[u8],
    ) -> u32 {
        if !self.feature_enabled(VIRTIO_GPU_F_RESOURCE_BLOB) {
            return RESP_ERR_UNSPEC;
        }
        let Some(resource_id) = unmap_id(input) else {
            return RESP_ERR_INVALID_PARAMETER;
        };
        let Some(blob) = self.blobs.get_mut(&resource_id) else {
            return RESP_ERR_INVALID_RESOURCE_ID;
        };
        let Some(host) = blob.host.as_mut() else {
            return RESP_ERR_INVALID_PARAMETER;
        };
        let Some(offset) = host.mapped_offset else {
            return RESP_ERR_INVALID_PARAMETER;
        };
        let Some(address) = aperture_address(offset, blob.size) else {
            return RESP_ERR_INVALID_PARAMETER;
        };
        if mem.read_bytes(address, &mut host.bytes).is_none()
            || mem.discard_range(address, blob.size).is_none()
        {
            return RESP_ERR_UNSPEC;
        }
        host.mapped_offset = None;
        RESP_OK_NODATA
    }

    fn map_blob_inner(&mut self, mem: &mut PhysicalMemory, input: &[u8]) -> Result<(), u32> {
        if !self.feature_enabled(VIRTIO_GPU_F_RESOURCE_BLOB) {
            return Err(RESP_ERR_UNSPEC);
        }
        let Some((resource_id, offset)) = map_request(input) else {
            return Err(RESP_ERR_INVALID_PARAMETER);
        };
        let Some(blob) = self.blobs.get(&resource_id) else {
            return Err(RESP_ERR_INVALID_RESOURCE_ID);
        };
        let Some(host) = blob.host.as_ref() else {
            return Err(RESP_ERR_INVALID_PARAMETER);
        };
        let Some(address) = aperture_address(offset, blob.size) else {
            return Err(RESP_ERR_INVALID_PARAMETER);
        };
        if host.mapped_offset.is_some()
            || !self.virgl_contexts.contains_key(&host.owner_context)
            || self.mapping_conflicts(resource_id, offset, blob.size)
        {
            return Err(RESP_ERR_INVALID_PARAMETER);
        }
        let blob = self
            .blobs
            .get_mut(&resource_id)
            .expect("blob checked above");
        let host = blob.host.as_mut().expect("host blob checked above");
        mem.write_bytes(address, &host.bytes)
            .ok_or(RESP_ERR_UNSPEC)?;
        host.mapped_offset = Some(offset);
        Ok(())
    }

    fn mapping_conflicts(&self, resource_id: u32, offset: u64, size: usize) -> bool {
        self.blobs.iter().any(|(id, blob)| {
            *id != resource_id
                && blob
                    .mapped_range()
                    .is_some_and(|(other, len)| ranges_overlap(offset, size, other, len))
        })
    }
}

fn map_request(input: &[u8]) -> Option<(u32, u64)> {
    (input.len() == 40 && read_u32(input, 28)? == 0)
        .then(|| Some((read_u32(input, 24)?, read_u64(input, 32)?)))?
}

fn unmap_id(input: &[u8]) -> Option<u32> {
    if input.len() == 32 && read_u32(input, 28)? == 0 {
        read_u32(input, 24)
    } else {
        None
    }
}

fn aperture_address(offset: u64, size: usize) -> Option<u64> {
    let size = u64::try_from(size).ok()?;
    (offset % PAGE_SIZE == 0 && offset.checked_add(size)? <= VIRTIO_GPU_HOST_VISIBLE_SIZE)
        .then_some(VIRTIO_GPU_HOST_VISIBLE_BASE + offset)
}

fn ranges_overlap(first: u64, first_len: usize, second: u64, second_len: usize) -> bool {
    let Some(first_end) = first.checked_add(first_len as u64) else {
        return true;
    };
    let Some(second_end) = second.checked_add(second_len as u64) else {
        return true;
    };
    first < second_end && second < first_end
}
