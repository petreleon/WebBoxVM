use super::super::VirglContext;
use super::decode::state::Command;
use crate::devices::virtio_gpu::protocol::RESP_ERR_INVALID_PARAMETER;

pub(super) fn apply(context: &mut VirglContext, command: Command) -> Result<(), u32> {
    match command {
        Command::CreateRasterizer { handle, scissor } => context
            .create_rasterizer(handle, scissor)
            .then_some(())
            .ok_or(RESP_ERR_INVALID_PARAMETER),
        Command::BindRasterizer { handle } => context
            .bind_rasterizer(handle)
            .then_some(())
            .ok_or(RESP_ERR_INVALID_PARAMETER),
        Command::DestroyRasterizer { handle } => context
            .destroy_rasterizer(handle)
            .then_some(())
            .ok_or(RESP_ERR_INVALID_PARAMETER),
        Command::Viewport(viewport) => {
            context.set_viewport(viewport);
            Ok(())
        }
        Command::Scissor(scissor) => {
            context.set_scissor(scissor);
            Ok(())
        }
    }
}
