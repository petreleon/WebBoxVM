use super::super::VirglContext;
use super::decode::sampler::Command;
use crate::devices::virtio_gpu::VirtioGpu;
use crate::devices::virtio_gpu::protocol::{
    RESP_ERR_INVALID_PARAMETER, RESP_ERR_INVALID_RESOURCE_ID,
};

pub(super) fn apply(
    gpu: &VirtioGpu,
    context: &mut VirglContext,
    command: Command,
) -> Result<(), u32> {
    match command {
        Command::CreateState { handle, state } => context
            .create_sampler_state(handle, state)
            .then_some(())
            .ok_or(RESP_ERR_INVALID_PARAMETER),
        Command::DestroyState { handle } => context
            .destroy_sampler_state(handle)
            .then_some(())
            .ok_or(RESP_ERR_INVALID_PARAMETER),
        Command::BindState { start, handles } => context
            .bind_sampler_states(start, &handles)
            .then_some(())
            .ok_or(RESP_ERR_INVALID_PARAMETER),
        Command::CreateView {
            handle,
            resource,
            format,
        } => create_view(gpu, context, handle, resource, format),
        Command::DestroyView { handle } => context
            .destroy_sampler_view(handle)
            .then_some(())
            .ok_or(RESP_ERR_INVALID_PARAMETER),
        Command::BindView { start, handles } => context
            .bind_sampler_views(start, &handles)
            .then_some(())
            .ok_or(RESP_ERR_INVALID_PARAMETER),
    }
}

fn create_view(
    gpu: &VirtioGpu,
    context: &mut VirglContext,
    handle: u32,
    resource: u32,
    format: u32,
) -> Result<(), u32> {
    let texture = gpu
        .resources
        .get(&resource)
        .ok_or(RESP_ERR_INVALID_RESOURCE_ID)?;
    if !context.is_attached(resource)
        || !gpu.is_virgl_resource(resource)
        || !texture.is_texture_2d()
        || !texture.is_sampled()
        || texture.format != format
        || !context.create_sampler_view(handle, resource)
    {
        return Err(RESP_ERR_INVALID_PARAMETER);
    }
    Ok(())
}
