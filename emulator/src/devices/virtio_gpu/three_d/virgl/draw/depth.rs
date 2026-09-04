use super::DrawMaterial;
use super::super::{DepthState, VirglContext};
use crate::devices::virtio_gpu::VirtioGpu;
use crate::devices::virtio_gpu::protocol::RESP_ERR_INVALID_PARAMETER;

pub(super) fn validate(
    gpu: &VirtioGpu,
    context: &VirglContext,
    color_resource: u32,
    depth_resource: Option<u32>,
    depth_state: Option<DepthState>,
    material: &DrawMaterial,
) -> Result<Option<u32>, u32> {
    match (depth_state, depth_resource) {
        (None, None) => Ok(None),
        (Some(_), Some(depth_resource)) => checked(gpu, context, color_resource, depth_resource, material),
        _ => Err(RESP_ERR_INVALID_PARAMETER),
    }
}

fn checked(
    gpu: &VirtioGpu,
    context: &VirglContext,
    color_resource: u32,
    depth_resource: u32,
    material: &DrawMaterial,
) -> Result<Option<u32>, u32> {
    let color = gpu.resources.get(&color_resource).ok_or(RESP_ERR_INVALID_PARAMETER)?;
    let depth = gpu.resources.get(&depth_resource).ok_or(RESP_ERR_INVALID_PARAMETER)?;
    if !matches!(material, DrawMaterial::Solid(_) | DrawMaterial::VertexColor | DrawMaterial::Textured(_))
        || context.framebuffer_depth_resource() != Some(depth_resource)
        || color_resource == depth_resource
        || !depth.is_depth_texture_2d()
        || depth.width != color.width
        || depth.height != color.height
    {
        return Err(RESP_ERR_INVALID_PARAMETER);
    }
    Ok(Some(depth_resource))
}
