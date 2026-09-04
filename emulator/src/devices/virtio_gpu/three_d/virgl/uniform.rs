use super::{FragmentConstants, UniformBinding, VirglContext};
use crate::devices::virtio_gpu::VirtioGpu;
use crate::devices::virtio_gpu::protocol::{
    RESP_ERR_INVALID_PARAMETER, RESP_ERR_INVALID_RESOURCE_ID,
};
use crate::devices::virtio_gpu::resource::BufferBind;

const BYTES: usize = 16;

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
    source: Option<UniformBinding>,
) -> Result<[f32; 2], u32> {
    let binding = source.ok_or(RESP_ERR_INVALID_PARAMETER)?;
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

pub(super) fn vertex_matrix(source: Option<[u32; 16]>) -> Result<[f32; 16], u32> {
    let matrix = source
        .ok_or(RESP_ERR_INVALID_PARAMETER)?
        .map(f32::from_bits);
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
                    .checked_add(BYTES)
                    .ok_or(RESP_ERR_INVALID_PARAMETER)?,
        )
        .ok_or(RESP_ERR_INVALID_PARAMETER)?;
    let word = |start| {
        bytes
            .get(start..start + 4)
            .and_then(|word| word.try_into().ok())
            .map(u32::from_le_bytes)
            .ok_or(RESP_ERR_INVALID_PARAMETER)
    };
    Ok([word(0)?, word(4)?, word(8)?, word(12)?])
}
