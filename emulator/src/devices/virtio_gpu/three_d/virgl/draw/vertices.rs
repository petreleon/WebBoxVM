use super::{DrawCall, DrawState};
use crate::devices::virtio_gpu::VirtioGpu;
use crate::devices::virtio_gpu::protocol::RESP_ERR_INVALID_PARAMETER;
use crate::devices::virtio_gpu::resource::{
    BufferBind, FORMAT_R8_UNORM, FORMAT_R32G32_FLOAT, FORMAT_R32G32B32A32_FLOAT, GpuResource,
};
use crate::devices::virtio_gpu::three_d::virgl::{
    VertexBuffer, VertexElement, VirglContext,
    context::MAX_VIRGL_VERTEX_BUFFERS,
};

#[derive(Clone, Copy)]
struct Source<'a> {
    binding: VertexBuffer,
    resource: &'a GpuResource,
}

pub(super) fn resolve(
    gpu: &VirtioGpu,
    context: &VirglContext,
    target: u32,
    state: DrawState,
    call: DrawCall,
    vertex_bytes: usize,
) -> Result<Vec<u8>, u32> {
    if state.vertex_layout.normalized_stride() != Some(vertex_bytes) {
        return Err(RESP_ERR_INVALID_PARAMETER);
    }
    let sources = sources(gpu, context, target, state)?;
    let indices = if call.indexed {
        index_values(gpu, context, state, call)?
    } else {
        sequential(call.start, call.count)?
    };
    let capacity = usize::try_from(call.count)
        .ok()
        .and_then(|count| count.checked_mul(vertex_bytes))
        .ok_or(RESP_ERR_INVALID_PARAMETER)?;
    let mut vertices = Vec::with_capacity(capacity);
    for index in indices {
        for element in state.vertex_layout.elements() {
            let slot = usize::try_from(element.buffer_index).map_err(|_| RESP_ERR_INVALID_PARAMETER)?;
            let source = sources[slot].ok_or(RESP_ERR_INVALID_PARAMETER)?;
            vertices.extend_from_slice(attribute(source, *element, index)?);
        }
    }
    Ok(vertices)
}

fn sources<'a>(
    gpu: &'a VirtioGpu,
    context: &VirglContext,
    target: u32,
    state: DrawState,
) -> Result<[Option<Source<'a>>; MAX_VIRGL_VERTEX_BUFFERS], u32> {
    let mut sources = [None; MAX_VIRGL_VERTEX_BUFFERS];
    for slot in 0..MAX_VIRGL_VERTEX_BUFFERS {
        let Some(stride) = state.vertex_layout.slot_stride(slot) else {
            continue;
        };
        let binding = state.vertex_buffers[slot].ok_or(RESP_ERR_INVALID_PARAMETER)?;
        let source = gpu
            .resources
            .get(&binding.resource)
            .ok_or(RESP_ERR_INVALID_PARAMETER)?;
        let offset = usize::try_from(binding.offset).map_err(|_| RESP_ERR_INVALID_PARAMETER)?;
        let valid = binding.stride == stride
            && binding.offset.is_multiple_of(stride)
            && state.vertex_layout.slot_format(slot) == Some(source.format)
            && source.is_buffer_bind(BufferBind::Vertex)
            && offset < source.pixels.len()
            && context.is_attached(target)
            && context.is_attached(binding.resource)
            && gpu.is_virgl_resource(target)
            && gpu.is_virgl_resource(binding.resource);
        if !valid {
            return Err(RESP_ERR_INVALID_PARAMETER);
        }
        sources[slot] = Some(Source { binding, resource: source });
    }
    Ok(sources)
}

fn sequential(start: u32, count: u32) -> Result<Vec<u32>, u32> {
    let end = start.checked_add(count).ok_or(RESP_ERR_INVALID_PARAMETER)?;
    Ok((start..end).collect())
}

fn index_values(
    gpu: &VirtioGpu,
    context: &VirglContext,
    state: DrawState,
    call: DrawCall,
) -> Result<Vec<u32>, u32> {
    let binding = state.index_buffer.ok_or(RESP_ERR_INVALID_PARAMETER)?;
    let source = gpu
        .resources
        .get(&binding.resource)
        .ok_or(RESP_ERR_INVALID_PARAMETER)?;
    let size = usize::try_from(binding.index_size).map_err(|_| RESP_ERR_INVALID_PARAMETER)?;
    let offset = usize::try_from(binding.offset).map_err(|_| RESP_ERR_INVALID_PARAMETER)?;
    let valid = matches!(binding.index_size, 2 | 4)
        && binding.offset.is_multiple_of(binding.index_size)
        && offset.checked_add(size).is_some_and(|end| end <= source.pixels.len())
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
    let count = usize::try_from(call.count).map_err(|_| RESP_ERR_INVALID_PARAMETER)?;
    let bytes = count.checked_mul(size).ok_or(RESP_ERR_INVALID_PARAMETER)?;
    let raw = source
        .pixels
        .get(start..start.checked_add(bytes).ok_or(RESP_ERR_INVALID_PARAMETER)?)
        .ok_or(RESP_ERR_INVALID_PARAMETER)?;
    raw.chunks_exact(size)
        .map(|bytes| index_value(bytes).ok_or(RESP_ERR_INVALID_PARAMETER))
        .collect()
}

fn index_value(bytes: &[u8]) -> Option<u32> {
    match bytes {
        [low, high] => Some(u32::from(u16::from_le_bytes([*low, *high]))),
        [a, b, c, d] => Some(u32::from_le_bytes([*a, *b, *c, *d])),
        _ => None,
    }
}

fn attribute<'a>(source: Source<'a>, element: VertexElement, index: u32) -> Result<&'a [u8], u32> {
    let offset = usize::try_from(source.binding.offset).map_err(|_| RESP_ERR_INVALID_PARAMETER)?;
    let stride = usize::try_from(source.binding.stride).map_err(|_| RESP_ERR_INVALID_PARAMETER)?;
    let index = usize::try_from(index).map_err(|_| RESP_ERR_INVALID_PARAMETER)?;
    let element_offset = usize::try_from(element.offset).map_err(|_| RESP_ERR_INVALID_PARAMETER)?;
    let bytes = element_bytes(element.format).ok_or(RESP_ERR_INVALID_PARAMETER)?;
    let start = index
        .checked_mul(stride)
        .and_then(|start| offset.checked_add(start))
        .and_then(|start| element_offset.checked_add(start))
        .ok_or(RESP_ERR_INVALID_PARAMETER)?;
    source
        .resource
        .pixels
        .get(start..start.checked_add(bytes).ok_or(RESP_ERR_INVALID_PARAMETER)?)
        .ok_or(RESP_ERR_INVALID_PARAMETER)
}

fn element_bytes(format: u32) -> Option<usize> {
    match format {
        FORMAT_R8_UNORM => Some(1),
        FORMAT_R32G32_FLOAT => Some(8),
        FORMAT_R32G32B32A32_FLOAT => Some(16),
        _ => None,
    }
}
