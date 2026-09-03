use super::super::SampledResource;
use super::TextureSnapshot;
use crate::devices::virtio_gpu::VirtioGpu;
use crate::devices::virtio_gpu::protocol::RESP_ERR_INVALID_PARAMETER;
use crate::devices::virtio_gpu::resource::sampled_texture_format;
use crate::devices::virtio_gpu::three_d::virgl::VirglContext;

const MAX_TEXTURE_DIMENSION: u32 = 64;

pub(super) fn snapshot(
    gpu: &VirtioGpu,
    context: &VirglContext,
    target: u32,
    resource: Option<SampledResource>,
) -> Result<TextureSnapshot, u32> {
    let resource = resource.ok_or(RESP_ERR_INVALID_PARAMETER)?;
    let texture = gpu
        .resources
        .get(&resource.resource)
        .ok_or(RESP_ERR_INVALID_PARAMETER)?;
    let pixels = usize::try_from(texture.width)
        .ok()
        .and_then(|width| width.checked_mul(texture.height as usize))
        .and_then(|pixels| pixels.checked_mul(4))
        .ok_or(RESP_ERR_INVALID_PARAMETER)?;
    let valid = resource.resource != target
        && context.is_attached(resource.resource)
        && gpu.is_virgl_resource(resource.resource)
        && texture.is_texture_2d()
        && texture.is_sampled()
        && sampled_texture_format(texture.format)
        && texture.width <= MAX_TEXTURE_DIMENSION
        && texture.height <= MAX_TEXTURE_DIMENSION
        && texture.pixels.len() == pixels;
    if !valid {
        return Err(RESP_ERR_INVALID_PARAMETER);
    }
    Ok(TextureSnapshot {
        width: texture.width,
        height: texture.height,
        bgra: texture.pixels.clone(),
        address_mode: resource.address_mode,
    })
}
