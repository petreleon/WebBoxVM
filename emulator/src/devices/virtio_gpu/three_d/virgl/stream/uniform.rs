use super::super::{UniformBinding, VirglContext, uniform};
use super::decode::uniform::Command;
use crate::devices::virtio_gpu::VirtioGpu;
use crate::devices::virtio_gpu::three_d::virgl::ShaderKind;

pub(super) fn apply(
    gpu: &VirtioGpu,
    context: &mut VirglContext,
    command: Command,
) -> Result<(), u32> {
    match command {
        Command::Clear(ShaderKind::Vertex) => context.set_vertex_uniform(None),
        Command::Clear(ShaderKind::Fragment) => context.set_fragment_constants(None),
        Command::Set {
            kind,
            resource,
            offset,
        } => {
            uniform::snapshot(gpu, context, resource, offset)?;
            match kind {
                ShaderKind::Vertex => {
                    context.set_vertex_uniform(Some(UniformBinding { resource, offset }))
                }
                ShaderKind::Fragment => context.set_fragment_uniform(resource, offset),
            }
        }
    }
    Ok(())
}
