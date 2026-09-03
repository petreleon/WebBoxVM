use super::super::{IndexBuffer, VirglContext};
use super::decode::index::Command;
use crate::devices::virtio_gpu::VirtioGpu;
use crate::devices::virtio_gpu::protocol::{
    RESP_ERR_INVALID_PARAMETER, RESP_ERR_INVALID_RESOURCE_ID,
};
use crate::devices::virtio_gpu::resource::BufferBind;

pub(super) fn apply(
    gpu: &VirtioGpu,
    context: &mut VirglContext,
    command: Command,
) -> Result<(), u32> {
    match command {
        Command::SetBuffer(binding) => set_buffer(gpu, context, binding),
    }
}

fn set_buffer(
    gpu: &VirtioGpu,
    context: &mut VirglContext,
    binding: Option<IndexBuffer>,
) -> Result<(), u32> {
    let Some(binding) = binding else {
        context.set_index_buffer(None);
        return Ok(());
    };
    if binding.resource == 0 || !matches!(binding.index_size, 2 | 4) {
        return Err(RESP_ERR_INVALID_PARAMETER);
    }
    let resource = gpu
        .resources
        .get(&binding.resource)
        .ok_or(RESP_ERR_INVALID_RESOURCE_ID)?;
    let size = usize::try_from(binding.index_size).map_err(|_| RESP_ERR_INVALID_PARAMETER)?;
    let offset = usize::try_from(binding.offset).map_err(|_| RESP_ERR_INVALID_PARAMETER)?;
    let valid = binding.offset.is_multiple_of(binding.index_size)
        && offset
            .checked_add(size)
            .is_some_and(|end| end <= resource.pixels.len())
        && context.is_attached(binding.resource)
        && gpu.is_virgl_resource(binding.resource)
        && resource.is_buffer_bind(BufferBind::Index);
    if !valid {
        return Err(RESP_ERR_INVALID_PARAMETER);
    }
    context.set_index_buffer(Some(binding));
    Ok(())
}
