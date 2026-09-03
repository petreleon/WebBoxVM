use super::{DrawCall, DrawState, TRIANGLE_VERTICES};
use crate::devices::virtio_gpu::VirtioGpu;
use crate::devices::virtio_gpu::protocol::RESP_ERR_INVALID_PARAMETER;
use crate::devices::virtio_gpu::resource::{BufferBind, FORMAT_R32G32B32A32_FLOAT, GpuResource};
use crate::devices::virtio_gpu::three_d::virgl::{VertexBuffer, VirglContext};

pub(super) fn resolve(
    gpu: &VirtioGpu,
    context: &VirglContext,
    target: u32,
    state: DrawState,
    call: DrawCall,
    vertex_bytes: usize,
) -> Result<Vec<u8>, u32> {
    let source = vertex_source(gpu, context, target, state, vertex_bytes)?;
    let indices = if call.indexed {
        index_values(gpu, context, state, call)?
    } else {
        sequential(call.start)?
    };
    let capacity = (TRIANGLE_VERTICES as usize)
        .checked_mul(vertex_bytes)
        .ok_or(RESP_ERR_INVALID_PARAMETER)?;
    let mut vertices = Vec::with_capacity(capacity);
    for index in indices {
        vertices.extend_from_slice(vertex(source, state.vertex_buffer, index, vertex_bytes)?);
    }
    Ok(vertices)
}

fn vertex_source<'a>(
    gpu: &'a VirtioGpu,
    context: &VirglContext,
    target: u32,
    state: DrawState,
    vertex_bytes: usize,
) -> Result<&'a GpuResource, u32> {
    let binding = state.vertex_buffer;
    let source = gpu
        .resources
        .get(&binding.resource)
        .ok_or(RESP_ERR_INVALID_PARAMETER)?;
    let stride = u32::try_from(vertex_bytes).map_err(|_| RESP_ERR_INVALID_PARAMETER)?;
    let valid = binding.stride == stride
        && binding.offset.is_multiple_of(stride)
        && state.vertex_layout.draw_stride() == Some(stride)
        && source.format == FORMAT_R32G32B32A32_FLOAT
        && source.is_buffer_bind(BufferBind::Vertex)
        && context.is_attached(target)
        && context.is_attached(binding.resource)
        && gpu.is_virgl_resource(target)
        && gpu.is_virgl_resource(binding.resource);
    valid.then_some(source).ok_or(RESP_ERR_INVALID_PARAMETER)
}

fn sequential(start: u32) -> Result<[u32; TRIANGLE_VERTICES as usize], u32> {
    Ok([
        start,
        start.checked_add(1).ok_or(RESP_ERR_INVALID_PARAMETER)?,
        start.checked_add(2).ok_or(RESP_ERR_INVALID_PARAMETER)?,
    ])
}

fn index_values(
    gpu: &VirtioGpu,
    context: &VirglContext,
    state: DrawState,
    call: DrawCall,
) -> Result<[u32; TRIANGLE_VERTICES as usize], u32> {
    let binding = state.index_buffer.ok_or(RESP_ERR_INVALID_PARAMETER)?;
    let source = gpu
        .resources
        .get(&binding.resource)
        .ok_or(RESP_ERR_INVALID_PARAMETER)?;
    let size = usize::try_from(binding.index_size).map_err(|_| RESP_ERR_INVALID_PARAMETER)?;
    let offset = usize::try_from(binding.offset).map_err(|_| RESP_ERR_INVALID_PARAMETER)?;
    let valid = matches!(binding.index_size, 2 | 4)
        && binding.offset.is_multiple_of(binding.index_size)
        && offset
            .checked_add(size)
            .is_some_and(|end| end <= source.pixels.len())
        && source.is_buffer_bind(BufferBind::Index)
        && context.is_attached(binding.resource)
        && gpu.is_virgl_resource(binding.resource);
    if !valid {
        return Err(RESP_ERR_INVALID_PARAMETER);
    }
    let start = usize::try_from(call.start)
        .ok()
        .and_then(|start| start.checked_mul(size))
        .and_then(|start| offset.checked_add(start))
        .ok_or(RESP_ERR_INVALID_PARAMETER)?;
    let bytes = (TRIANGLE_VERTICES as usize)
        .checked_mul(size)
        .ok_or(RESP_ERR_INVALID_PARAMETER)?;
    let raw = source
        .pixels
        .get(start..start.checked_add(bytes).ok_or(RESP_ERR_INVALID_PARAMETER)?)
        .ok_or(RESP_ERR_INVALID_PARAMETER)?;
    let mut values = [0; TRIANGLE_VERTICES as usize];
    for (value, bytes) in values.iter_mut().zip(raw.chunks_exact(size)) {
        *value = index_value(bytes).ok_or(RESP_ERR_INVALID_PARAMETER)?;
    }
    Ok(values)
}

fn index_value(bytes: &[u8]) -> Option<u32> {
    match bytes {
        [low, high] => Some(u32::from(u16::from_le_bytes([*low, *high]))),
        [a, b, c, d] => Some(u32::from_le_bytes([*a, *b, *c, *d])),
        _ => None,
    }
}

fn vertex(
    source: &GpuResource,
    binding: VertexBuffer,
    index: u32,
    bytes: usize,
) -> Result<&[u8], u32> {
    let offset = usize::try_from(binding.offset).map_err(|_| RESP_ERR_INVALID_PARAMETER)?;
    let index = usize::try_from(index).map_err(|_| RESP_ERR_INVALID_PARAMETER)?;
    let start = index
        .checked_mul(bytes)
        .and_then(|start| offset.checked_add(start))
        .ok_or(RESP_ERR_INVALID_PARAMETER)?;
    source
        .pixels
        .get(start..start.checked_add(bytes).ok_or(RESP_ERR_INVALID_PARAMETER)?)
        .ok_or(RESP_ERR_INVALID_PARAMETER)
}
