use super::super::{
    VertexBuffer, VirglContext,
    context::{MAX_VIRGL_VERTEX_BUFFERS, VertexLayout},
};
use super::decode::vertex::Command;
use crate::devices::virtio_gpu::VirtioGpu;
use crate::devices::virtio_gpu::protocol::{
    RESP_ERR_INVALID_PARAMETER, RESP_ERR_INVALID_RESOURCE_ID,
};
use crate::devices::virtio_gpu::resource::{
    BufferBind, FORMAT_R8_UNORM, FORMAT_R32G32_FLOAT, FORMAT_R32G32B32A32_FLOAT,
};

pub(super) fn apply(
    gpu: &VirtioGpu,
    context: &mut VirglContext,
    command: Command,
) -> Result<(), u32> {
    match command {
        Command::Create { handle, layout } => create(context, handle, layout),
        Command::Bind { handle } => context
            .bind_vertex_elements(handle)
            .then_some(())
            .ok_or(RESP_ERR_INVALID_PARAMETER),
        Command::Destroy { handle } => context
            .destroy_vertex_elements(handle)
            .then_some(())
            .ok_or(RESP_ERR_INVALID_PARAMETER),
        Command::SetBuffers(bindings) => set_buffers(gpu, context, bindings),
    }
}

fn create(context: &mut VirglContext, handle: u32, layout: VertexLayout) -> Result<(), u32> {
    if !layout.valid() || !context.create_vertex_elements(handle, layout) {
        return Err(RESP_ERR_INVALID_PARAMETER);
    }
    Ok(())
}

fn set_buffers(
    gpu: &VirtioGpu,
    context: &mut VirglContext,
    bindings: [Option<VertexBuffer>; MAX_VIRGL_VERTEX_BUFFERS],
) -> Result<(), u32> {
    for binding in bindings.into_iter().flatten() {
        let resource = gpu
            .resources
            .get(&binding.resource)
            .ok_or(RESP_ERR_INVALID_RESOURCE_ID)?;
        let offset = usize::try_from(binding.offset).map_err(|_| RESP_ERR_INVALID_PARAMETER)?;
        let shape = matches!(
            (binding.stride, resource.format),
            (1, FORMAT_R8_UNORM)
                | (8, FORMAT_R32G32_FLOAT)
                | (16 | 24 | 32 | 40, FORMAT_R32G32B32A32_FLOAT)
        );
        if !shape
            || !context.is_attached(binding.resource)
            || !gpu.is_virgl_resource(binding.resource)
            || !resource.is_buffer_bind(BufferBind::Vertex)
            || !binding.offset.is_multiple_of(binding.stride)
            || offset >= resource.pixels.len()
        {
            return Err(RESP_ERR_INVALID_PARAMETER);
        }
    }
    context.set_vertex_buffers(bindings);
    Ok(())
}
