use super::{FragmentConstants, VertexConstants, VirglContext};
use crate::devices::virtio_gpu::VirtioGpu;
use crate::devices::virtio_gpu::protocol::{
    RESP_ERR_INVALID_PARAMETER, RESP_ERR_INVALID_RESOURCE_ID,
};
use crate::devices::virtio_gpu::resource::BufferBind;

pub(super) fn resolve(
    gpu: &VirtioGpu,
    context: &VirglContext,
    source: Option<FragmentConstants>,
) -> Result<[u32; 4], u32> {
    match source.ok_or(RESP_ERR_INVALID_PARAMETER)? {
        FragmentConstants::Inline(values) => Ok(values),
        FragmentConstants::Uniform(binding) => {
            snapshot(gpu, context, binding.resource, binding.offset)
        }
    }
}

pub(super) fn vertex_offset(
    gpu: &VirtioGpu,
    context: &VirglContext,
    source: Option<VertexConstants>,
) -> Result<[f32; 2], u32> {
    let binding = match source {
        Some(VertexConstants::UniformOffset(binding)) => binding,
        _ => return Err(RESP_ERR_INVALID_PARAMETER),
    };
    let [x, y, z, w] =
        snapshot(gpu, context, binding.resource, binding.offset)?.map(f32::from_bits);
    (x.is_finite()
        && y.is_finite()
        && (-1.0..=1.0).contains(&x)
        && (-1.0..=1.0).contains(&y)
        && z == 0.0
        && w == 0.0)
        .then_some([x, y])
        .ok_or(RESP_ERR_INVALID_PARAMETER)
}

pub(super) fn vertex_matrix(
    gpu: &VirtioGpu,
    context: &VirglContext,
    source: Option<VertexConstants>,
) -> Result<[f32; 16], u32> {
    let values = match source {
        Some(VertexConstants::InlineMatrix(values)) => values,
        Some(VertexConstants::UniformMatrix(binding)) => {
            matrix_snapshot(gpu, context, binding.resource, binding.offset)?
        }
        _ => return Err(RESP_ERR_INVALID_PARAMETER),
    };
    let matrix = values.map(f32::from_bits);
    matrix
        .iter()
        .all(|value| value.is_finite())
        .then_some(matrix)
        .ok_or(RESP_ERR_INVALID_PARAMETER)
}

pub(super) fn snapshot(
    gpu: &VirtioGpu,
    context: &VirglContext,
    resource_id: u32,
    offset: u32,
) -> Result<[u32; 4], u32> {
    words(gpu, context, resource_id, offset)
}

pub(super) fn matrix_snapshot(
    gpu: &VirtioGpu,
    context: &VirglContext,
    resource_id: u32,
    offset: u32,
) -> Result<[u32; 16], u32> {
    words(gpu, context, resource_id, offset)
}

fn words<const COUNT: usize>(
    gpu: &VirtioGpu,
    context: &VirglContext,
    resource_id: u32,
    offset: u32,
) -> Result<[u32; COUNT], u32> {
    let resource = gpu
        .resources
        .get(&resource_id)
        .ok_or(RESP_ERR_INVALID_RESOURCE_ID)?;
    let offset = usize::try_from(offset).map_err(|_| RESP_ERR_INVALID_PARAMETER)?;
    if !context.is_attached(resource_id)
        || !gpu.is_virgl_resource(resource_id)
        || !resource.is_buffer_bind(BufferBind::Uniform)
        || offset % 4 != 0
    {
        return Err(RESP_ERR_INVALID_PARAMETER);
    }
    let bytes = resource
        .pixels
        .get(
            offset
                ..offset
                    .checked_add(COUNT.checked_mul(4).ok_or(RESP_ERR_INVALID_PARAMETER)?)
                    .ok_or(RESP_ERR_INVALID_PARAMETER)?,
        )
        .ok_or(RESP_ERR_INVALID_PARAMETER)?;
    let mut words = [0; COUNT];
    for (word, bytes) in words.iter_mut().zip(bytes.chunks_exact(4)) {
        *word = u32::from_le_bytes(bytes.try_into().map_err(|_| RESP_ERR_INVALID_PARAMETER)?);
    }
    Ok(words)
}
