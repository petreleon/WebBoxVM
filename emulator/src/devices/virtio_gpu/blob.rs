use super::backing::decode_entries;
use super::protocol::*;
use super::resource::total_resource_limit;
use super::{
    BackingEntry, MAX_BACKING_ENTRIES, MAX_RESOURCE_BYTES, MAX_RESOURCES, VIRTIO_GPU_F_RESOURCE_BLOB,
    VirtioGpu,
};
use crate::memory::PhysicalMemory;

const BLOB_MEM_GUEST: u32 = 1;

#[derive(Debug, Clone)]
pub(super) struct BlobResource {
    pub size: usize,
    pub backing: Vec<BackingEntry>,
}

impl VirtioGpu {
    pub(super) fn create_blob(&mut self, mem: &PhysicalMemory, input: &[u8]) -> u32 {
        if !self.feature_enabled(VIRTIO_GPU_F_RESOURCE_BLOB) {
            return RESP_ERR_UNSPEC;
        }
        let Some(create) = BlobCreate::decode(input) else {
            return RESP_ERR_INVALID_PARAMETER;
        };
        if create.resource_id == 0 || self.resource_exists(create.resource_id) {
            return RESP_ERR_INVALID_RESOURCE_ID;
        }
        if self.resource_count() >= MAX_RESOURCES {
            return RESP_ERR_OUT_OF_MEMORY;
        }
        let Some(backing) = blob_backing(mem, input, create.entries, create.size) else {
            return RESP_ERR_INVALID_PARAMETER;
        };
        if !total_resource_limit(self.allocated_resource_bytes, create.size) {
            return RESP_ERR_OUT_OF_MEMORY;
        }
        self.allocated_resource_bytes += create.size;
        self.blobs.insert(
            create.resource_id,
            BlobResource {
                size: create.size,
                backing,
            },
        );
        RESP_OK_NODATA
    }
}

struct BlobCreate {
    resource_id: u32,
    entries: usize,
    size: usize,
}

impl BlobCreate {
    fn decode(input: &[u8]) -> Option<Self> {
        if input.len() < 56
            || read_u32(input, 28)? != BLOB_MEM_GUEST
            || read_u32(input, 32)? != 0
            || read_u64(input, 40)? != 0
        {
            return None;
        }
        let entries = usize::try_from(read_u32(input, 36)?).ok()?;
        let size = usize::try_from(read_u64(input, 48)?).ok()?;
        let expected = entries.checked_mul(16)?.checked_add(56)?;
        (entries <= MAX_BACKING_ENTRIES && size != 0 && size <= MAX_RESOURCE_BYTES
            && input.len() == expected)
            .then_some(Self {
                resource_id: read_u32(input, 24)?,
                entries,
                size,
            })
    }
}

fn blob_backing(
    mem: &PhysicalMemory,
    input: &[u8],
    count: usize,
    size: usize,
) -> Option<Vec<BackingEntry>> {
    (count == 0)
        .then(Vec::new)
        .or_else(|| decode_entries(mem, input, 56, count, size))
}
