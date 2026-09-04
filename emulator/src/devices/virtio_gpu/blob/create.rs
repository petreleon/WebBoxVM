use super::super::backing::decode_entries;
use super::super::protocol::*;
use super::super::resource::total_resource_limit;
use super::super::{
    MAX_BACKING_ENTRIES, MAX_RESOURCE_BYTES, MAX_RESOURCES, VIRTIO_GPU_F_RESOURCE_BLOB,
    VIRTIO_GPU_F_VIRGL, VirtioGpu,
};
use super::{BLOB_FLAG_USE_MAPPABLE, BLOB_MEM_GUEST, BLOB_MEM_HOST3D, BLOB_MEM_HOST3D_GUEST};
use super::{BlobMemory, BlobResource};
use crate::constants::PAGE_SIZE;
use crate::memory::PhysicalMemory;

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
                if !self.feature_enabled(VIRTIO_GPU_F_VIRGL)
                    || !self.virgl_contexts.contains_key(&header.ctx_id)
                    || !self.renderer_blob_ready(header.ctx_id, &create, BlobMemory::Host3d)
                {
                    return RESP_ERR_INVALID_PARAMETER;
                }
                let Some(blob) = BlobResource::host_only(create.size, header.ctx_id) else {
                    return RESP_ERR_OUT_OF_MEMORY;
                };
                blob
            }
            BlobKind::Host3dGuest => {
                if !self.feature_enabled(VIRTIO_GPU_F_VIRGL)
                    || !self.virgl_contexts.contains_key(&header.ctx_id)
                    || !self.renderer_blob_ready(header.ctx_id, &create, BlobMemory::Host3dGuest)
                {
                    return RESP_ERR_INVALID_PARAMETER;
                }
                let Some(backing) = blob_backing(mem, input, create.entries, create.size) else {
                    return RESP_ERR_INVALID_PARAMETER;
                };
                let Some(blob) = BlobResource::host_shadowed(create.size, header.ctx_id, backing)
                else {
                    return RESP_ERR_OUT_OF_MEMORY;
                };
                blob
            }
        };
        if create.blob_id != 0 {
            self.virgl_contexts
                .get_mut(&header.ctx_id)
                .expect("context checked before blob allocation")
                .consume_renderer_blob(create.blob_id);
        }
        self.allocated_resource_bytes += create.size;
        self.blobs.insert(create.resource_id, blob);
        RESP_OK_NODATA
    }

    fn renderer_blob_ready(
        &self,
        context_id: u32,
        create: &BlobCreate,
        memory: BlobMemory,
    ) -> bool {
        create.blob_id == 0
            || self.virgl_contexts.get(&context_id).is_some_and(|context| {
                context.has_renderer_blob(create.blob_id, memory, create.flags, create.size)
            })
    }
}

#[derive(Clone, Copy)]
enum BlobKind {
    Guest,
    Host3d,
    Host3dGuest,
}

struct BlobCreate {
    resource_id: u32,
    entries: usize,
    size: usize,
    flags: u32,
    blob_id: u64,
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
        let memory = read_u32(input, 28)?;
        let flags = read_u32(input, 32)?;
        let blob_id = read_u64(input, 40)?;
        let kind = match (memory, flags, blob_id) {
            (BLOB_MEM_GUEST, 0, 0) => BlobKind::Guest,
            (BLOB_MEM_HOST3D, BLOB_FLAG_USE_MAPPABLE, _)
                if entries == 0 && size % PAGE_SIZE as usize == 0 =>
            {
                BlobKind::Host3d
            }
            (BLOB_MEM_HOST3D_GUEST, 0, _) if size % PAGE_SIZE as usize == 0 => {
                BlobKind::Host3dGuest
            }
            _ => return None,
        };
        Some(Self {
            resource_id: read_u32(input, 24)?,
            entries,
            size,
            flags,
            blob_id,
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
