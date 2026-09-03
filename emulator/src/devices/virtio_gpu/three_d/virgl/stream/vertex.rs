use super::super::{VertexBuffer, VertexElement, VirglContext};
use super::decode::vertex::Command;
use crate::devices::virtio_gpu::VirtioGpu;
use crate::devices::virtio_gpu::protocol::{
    RESP_ERR_INVALID_PARAMETER, RESP_ERR_INVALID_RESOURCE_ID,
};
use crate::devices::virtio_gpu::resource::FORMAT_R8_UNORM;

pub(super) fn apply(
    gpu: &VirtioGpu,
    context: &mut VirglContext,
    command: Command,
) -> Result<(), u32> {
    match command {
        Command::Create { handle, element } => create(context, handle, element),
        Command::Bind { handle } => context
            .bind_vertex_elements(handle)
            .then_some(())
            .ok_or(RESP_ERR_INVALID_PARAMETER),
        Command::Destroy { handle } => context
            .destroy_vertex_elements(handle)
            .then_some(())
            .ok_or(RESP_ERR_INVALID_PARAMETER),
        Command::SetBuffer(binding) => set_buffer(gpu, context, binding),
    }
}

fn create(context: &mut VirglContext, handle: u32, element: VertexElement) -> Result<(), u32> {
    let valid = element.offset == 0
        && element.divisor == 0
        && element.buffer_index == 0
        && element.format == FORMAT_R8_UNORM;
    if !valid || !context.create_vertex_elements(handle, element) {
        return Err(RESP_ERR_INVALID_PARAMETER);
    }
    Ok(())
}

fn set_buffer(
    gpu: &VirtioGpu,
    context: &mut VirglContext,
    binding: Option<VertexBuffer>,
) -> Result<(), u32> {
    let Some(binding) = binding else {
        context.set_vertex_buffer(None);
        return Ok(());
    };
    if binding.resource == 0 {
        if binding.stride != 0 || binding.offset != 0 {
            return Err(RESP_ERR_INVALID_PARAMETER);
        }
        context.set_vertex_buffer(None);
        return Ok(());
    }
    let resource = gpu
        .resources
        .get(&binding.resource)
        .ok_or(RESP_ERR_INVALID_RESOURCE_ID)?;
    let offset = usize::try_from(binding.offset).map_err(|_| RESP_ERR_INVALID_PARAMETER)?;
    if binding.stride != 1
        || !context.is_attached(binding.resource)
        || !gpu.is_virgl_resource(binding.resource)
        || !resource.is_buffer()
        || resource.format != FORMAT_R8_UNORM
        || offset >= resource.pixels.len()
    {
        return Err(RESP_ERR_INVALID_PARAMETER);
    }
    context.set_vertex_buffer(Some(binding));
    Ok(())
}
