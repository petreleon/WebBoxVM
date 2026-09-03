use super::super::VirglContext;
use super::decode::sampler::Command;
use crate::devices::virtio_gpu::VirtioGpu;
use crate::devices::virtio_gpu::protocol::{
    RESP_ERR_INVALID_PARAMETER, RESP_ERR_INVALID_RESOURCE_ID,
};
use crate::devices::virtio_gpu::resource::FORMAT_B8G8R8A8_UNORM;

pub(super) fn apply(
    gpu: &VirtioGpu,
    context: &mut VirglContext,
    command: Command,
) -> Result<(), u32> {
    match command {
        Command::CreateState { handle } => context
            .create_sampler_state(handle)
            .then_some(())
            .ok_or(RESP_ERR_INVALID_PARAMETER),
        Command::DestroyState { handle } => context
            .destroy_sampler_state(handle)
            .then_some(())
            .ok_or(RESP_ERR_INVALID_PARAMETER),
        Command::BindState { handle } => context
            .bind_sampler_state(handle)
            .then_some(())
            .ok_or(RESP_ERR_INVALID_PARAMETER),
        Command::CreateView { handle, resource } => create_view(gpu, context, handle, resource),
        Command::DestroyView { handle } => context
            .destroy_sampler_view(handle)
            .then_some(())
            .ok_or(RESP_ERR_INVALID_PARAMETER),
        Command::BindView { handle } => context
            .bind_sampler_view(handle)
            .then_some(())
            .ok_or(RESP_ERR_INVALID_PARAMETER),
    }
}

fn create_view(
    gpu: &VirtioGpu,
    context: &mut VirglContext,
    handle: u32,
    resource: u32,
) -> Result<(), u32> {
    let texture = gpu
        .resources
        .get(&resource)
        .ok_or(RESP_ERR_INVALID_RESOURCE_ID)?;
    if !context.is_attached(resource)
        || !gpu.is_virgl_resource(resource)
        || !texture.is_texture_2d()
        || !texture.is_sampled()
        || texture.format != FORMAT_B8G8R8A8_UNORM
        || !context.create_sampler_view(handle, resource)
    {
        return Err(RESP_ERR_INVALID_PARAMETER);
    }
    Ok(())
}
