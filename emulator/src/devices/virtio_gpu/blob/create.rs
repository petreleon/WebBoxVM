use super::super::backing::decode_entries;
use super::super::protocol::*;
use super::super::resource::total_resource_limit;
use super::super::{
    MAX_BACKING_ENTRIES, MAX_RESOURCE_BYTES, MAX_RESOURCES, VIRTIO_GPU_F_RESOURCE_BLOB, VirtioGpu,
};
use super::BlobResource;
use crate::constants::PAGE_SIZE;
use crate::memory::PhysicalMemory;

const BLOB_MEM_GUEST: u32 = 1;
const BLOB_MEM_HOST3D: u32 = 2;
const BLOB_FLAG_USE_MAPPABLE: u32 = 1;

impl VirtioGpu {
    pub(in crate::devices::virtio_gpu) fn create_blob(
        &mut self,
        mem: &PhysicalMemory,
        header: CtrlHeader,
        input: &[u8],
    ) -> u32 {
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
        if !total_resource_limit(self.allocated_resource_bytes, create.size) {
            return RESP_ERR_OUT_OF_MEMORY;
        }
        let blob = match create.kind {
            BlobKind::Guest => match blob_backing(mem, input, create.entries, create.size) {
                Some(backing) => BlobResource::guest(create.size, backing),
                None => return RESP_ERR_INVALID_PARAMETER,
            },
            BlobKind::Host3d => {
                if !self.virgl_contexts.contains_key(&header.ctx_id) {
                    return RESP_ERR_INVALID_PARAMETER;
                }
                let Some(blob) = BlobResource::host_visible(create.size, header.ctx_id) else {
                    return RESP_ERR_OUT_OF_MEMORY;
                };
                blob
            }
        };
        self.allocated_resource_bytes += create.size;
        self.blobs.insert(create.resource_id, blob);
        RESP_OK_NODATA
    }
}

#[derive(Clone, Copy)]
enum BlobKind {
    Guest,
    Host3d,
}

struct BlobCreate {
    resource_id: u32,
    entries: usize,
    size: usize,
    kind: BlobKind,
}

impl BlobCreate {
    fn decode(input: &[u8]) -> Option<Self> {
        if input.len() < 56 {
            return None;
        }
        let entries = usize::try_from(read_u32(input, 36)?).ok()?;
        let size = usize::try_from(read_u64(input, 48)?).ok()?;
        let expected = entries.checked_mul(16)?.checked_add(56)?;
        if entries > MAX_BACKING_ENTRIES
            || size == 0
            || size > MAX_RESOURCE_BYTES
            || input.len() != expected
        {
            return None;
        }
        let kind = match (
            read_u32(input, 28)?,
            read_u32(input, 32)?,
            read_u64(input, 40)?,
        ) {
            (BLOB_MEM_GUEST, 0, 0) => BlobKind::Guest,
            (BLOB_MEM_HOST3D, BLOB_FLAG_USE_MAPPABLE, 0)
                if entries == 0 && size % PAGE_SIZE as usize == 0 =>
            {
                BlobKind::Host3d
            }
            _ => return None,
        };
        Some(Self {
            resource_id: read_u32(input, 24)?,
            entries,
            size,
            kind,
        })
    }
}

fn blob_backing(
    mem: &PhysicalMemory,
    input: &[u8],
    count: usize,
    size: usize,
) -> Option<Vec<super::super::BackingEntry>> {
    (count == 0)
        .then(Vec::new)
        .or_else(|| decode_entries(mem, input, 56, count, size))
}
