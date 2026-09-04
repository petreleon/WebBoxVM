use super::super::{VirglContext, uniform};
use super::decode::uniform::Command;
use crate::devices::virtio_gpu::VirtioGpu;

pub(super) fn apply(
    gpu: &VirtioGpu,
    context: &mut VirglContext,
    command: Command,
) -> Result<(), u32> {
    match command {
        Command::Clear => context.set_fragment_constants(None),
        Command::SetFragment { resource, offset } => {
            uniform::snapshot(gpu, context, resource, offset)?;
            context.set_fragment_uniform(resource, offset);
        }
    }
    Ok(())
}
