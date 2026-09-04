use super::VirglContext;
use crate::constants::PAGE_SIZE;
use crate::devices::virtio_gpu::blob::{
    BLOB_FLAG_USE_MAPPABLE, BLOB_MEM_HOST3D, BLOB_MEM_HOST3D_GUEST, BlobMemory,
};
use crate::devices::virtio_gpu::protocol::{RESP_ERR_INVALID_PARAMETER, read_u32, read_u64};
use crate::devices::virtio_gpu::{MAX_RESOURCE_BYTES, MAX_RESOURCES};

const MAGIC: &[u8; 4] = b"WBL1";
const VERSION: u32 = 1;
const PACKET_BYTES: usize = 32;
pub(super) const MAX_RENDERER_BLOB_OBJECTS: usize = MAX_RESOURCES;

#[derive(Clone, Copy, Debug)]
pub(super) struct RendererBlobObject {
    memory: BlobMemory,
    flags: u32,
    size: usize,
}

impl RendererBlobObject {
    pub(super) fn matches(&self, memory: BlobMemory, flags: u32, size: usize) -> bool {
        self.memory == memory && self.flags == flags && self.size == size
    }
}

pub(super) fn prepare(context: &mut VirglContext, input: &[u8]) -> Option<Result<(), u32>> {
    if input.get(32..36) != Some(MAGIC) {
        return None;
    }
    let result = (|| {
        let (Some(payload_size), Some(padding), Some(version)) = (
            read_u32(input, 24),
            read_u32(input, 28),
            read_u32(input, 36),
        ) else {
            return Err(RESP_ERR_INVALID_PARAMETER);
        };
        if payload_size != PACKET_BYTES as u32
            || padding != 0
            || input.len() != 32 + PACKET_BYTES
            || version != VERSION
        {
            return Err(RESP_ERR_INVALID_PARAMETER);
        }
        let blob_id = read_u64(input, 40)
            .filter(|id| *id != 0)
            .ok_or(RESP_ERR_INVALID_PARAMETER)?;
        let Some(raw_size) = read_u64(input, 48) else {
            return Err(RESP_ERR_INVALID_PARAMETER);
        };
        let size = usize::try_from(raw_size).map_err(|_| RESP_ERR_INVALID_PARAMETER)?;
        let (Some(memory), Some(flags)) = (read_u32(input, 56), read_u32(input, 60)) else {
            return Err(RESP_ERR_INVALID_PARAMETER);
        };
        let (memory, flags) = match (memory, flags) {
            (BLOB_MEM_HOST3D, BLOB_FLAG_USE_MAPPABLE) => {
                (BlobMemory::Host3d, BLOB_FLAG_USE_MAPPABLE)
            }
            (BLOB_MEM_HOST3D_GUEST, 0) => (BlobMemory::Host3dGuest, 0),
            _ => return Err(RESP_ERR_INVALID_PARAMETER),
        };
        if size == 0 || size > MAX_RESOURCE_BYTES || size % PAGE_SIZE as usize != 0 {
            return Err(RESP_ERR_INVALID_PARAMETER);
        }
        context
            .prepare_renderer_blob(
                blob_id,
                RendererBlobObject {
                    memory,
                    flags,
                    size,
                },
            )
            .then_some(())
            .ok_or(RESP_ERR_INVALID_PARAMETER)
    })();
    Some(result)
}
