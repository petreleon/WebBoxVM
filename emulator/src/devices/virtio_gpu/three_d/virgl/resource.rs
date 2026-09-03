use crate::devices::virtio_gpu::protocol::*;
use crate::devices::virtio_gpu::resource::{GpuResource, total_resource_limit};
use crate::devices::virtio_gpu::{MAX_RESOURCES, VirtioGpu};

const VIRGL_TARGET_TEXTURE_2D: u32 = 2;
const VIRGL_BIND_RENDER_TARGET: u32 = 1 << 1;

impl VirtioGpu {
    pub(in crate::devices::virtio_gpu) fn create_virgl_resource(&mut self, input: &[u8]) -> u32 {
        let Some((id, target, format, bind, width, height, depth, array, level, samples, flags)) =
            decode_create(input)
        else {
            return RESP_ERR_INVALID_PARAMETER;
        };
        if id == 0 || self.resources.contains_key(&id) {
            return RESP_ERR_INVALID_RESOURCE_ID;
        }
        if self.resources.len() >= MAX_RESOURCES {
            return RESP_ERR_OUT_OF_MEMORY;
        }
        if target != VIRGL_TARGET_TEXTURE_2D
            || !GpuResource::supported_format(format)
            || bind != VIRGL_BIND_RENDER_TARGET
            || depth != 1
            || array != 1
            || level != 0
            || !matches!(samples, 0 | 1)
            || flags != 0
        {
            return RESP_ERR_INVALID_PARAMETER;
        }
        let Some(resource) = GpuResource::new(format, width, height) else {
            return RESP_ERR_INVALID_PARAMETER;
        };
        let bytes = resource.pixels.len();
        if !total_resource_limit(self.allocated_resource_bytes, bytes) {
            return RESP_ERR_OUT_OF_MEMORY;
        }
        self.allocated_resource_bytes += bytes;
        self.resources.insert(id, resource);
        self.virgl_resources.insert(id);
        RESP_OK_NODATA
    }
}

fn decode_create(input: &[u8]) -> Option<(u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32)> {
    if input.len() != 72 || read_u32(input, 68) != Some(0) {
        return None;
    }
    let values: Vec<u32> = (0..11)
        .map(|index| read_u32(input, 24 + index * 4))
        .collect::<Option<_>>()?;
    Some((
        values[0], values[1], values[2], values[3], values[4], values[5], values[6], values[7],
        values[8], values[9], values[10],
    ))
}
