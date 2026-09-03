use crate::devices::virtio_gpu::protocol::*;
use crate::devices::virtio_gpu::resource::{
    BufferBind, FORMAT_B8G8R8A8_UNORM, FORMAT_R8_UNORM, FORMAT_R32G32B32A32_FLOAT, GpuResource,
    sampled_texture_format, total_resource_limit,
};
use crate::devices::virtio_gpu::{MAX_RESOURCES, VirtioGpu};

const VIRGL_TARGET_BUFFER: u32 = 0;
const VIRGL_TARGET_TEXTURE_2D: u32 = 2;
const VIRGL_BIND_RENDER_TARGET: u32 = 1 << 1;
const VIRGL_BIND_SAMPLER_VIEW: u32 = 1 << 3;
const VIRGL_BIND_RENDER_AND_SAMPLE: u32 = VIRGL_BIND_RENDER_TARGET | VIRGL_BIND_SAMPLER_VIEW;
const VIRGL_BIND_VERTEX_BUFFER: u32 = 1 << 4;
const VIRGL_BIND_INDEX_BUFFER: u32 = 1 << 5;

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
        let resource = if let Some(sampleable) =
            color_texture(target, format, bind, depth, array, level, samples, flags)
        {
            GpuResource::new_texture(format, width, height, sampleable)
        } else if let Some(buffer_bind) = buffer_bind(
            target, format, bind, height, depth, array, level, samples, flags,
        ) {
            GpuResource::new_buffer(format, width, buffer_bind)
        } else {
            return RESP_ERR_INVALID_PARAMETER;
        };
        let Some(resource) = resource else {
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

fn color_texture(
    target: u32,
    format: u32,
    bind: u32,
    depth: u32,
    array: u32,
    level: u32,
    samples: u32,
    flags: u32,
) -> Option<bool> {
    (target == VIRGL_TARGET_TEXTURE_2D
        && GpuResource::virgl_texture_format(format)
        && depth == 1
        && array == 1
        && level == 0
        && matches!(samples, 0 | 1)
        && flags == 0)
        .then(|| match bind {
            VIRGL_BIND_RENDER_TARGET => Some(false),
            VIRGL_BIND_SAMPLER_VIEW if sampled_texture_format(format) => Some(true),
            VIRGL_BIND_RENDER_AND_SAMPLE if format == FORMAT_B8G8R8A8_UNORM => Some(true),
            _ => None,
        })
        .flatten()
}

fn buffer_bind(
    target: u32,
    format: u32,
    bind: u32,
    height: u32,
    depth: u32,
    array: u32,
    level: u32,
    samples: u32,
    flags: u32,
) -> Option<BufferBind> {
    (target == VIRGL_TARGET_BUFFER
        && height == 1
        && depth == 1
        && array == 1
        && level == 0
        && samples == 0
        && flags == 0)
        .then_some(match (format, bind) {
            (FORMAT_R8_UNORM | FORMAT_R32G32B32A32_FLOAT, VIRGL_BIND_VERTEX_BUFFER) => {
                Some(BufferBind::Vertex)
            }
            (FORMAT_R8_UNORM, VIRGL_BIND_INDEX_BUFFER) => Some(BufferBind::Index),
            _ => None,
        })
        .flatten()
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
