use super::super::VirglContext;
use super::decode::blend::Command;
use crate::devices::virtio_gpu::protocol::RESP_ERR_INVALID_PARAMETER;

pub(super) fn apply(context: &mut VirglContext, command: Command) -> Result<(), u32> {
    match command {
        Command::Create { handle } => context
            .create_blend(handle)
            .then_some(())
            .ok_or(RESP_ERR_INVALID_PARAMETER),
        Command::Bind { handle } => context
            .bind_blend(handle)
            .then_some(())
            .ok_or(RESP_ERR_INVALID_PARAMETER),
        Command::Destroy { handle } => context
            .destroy_blend(handle)
            .then_some(())
            .ok_or(RESP_ERR_INVALID_PARAMETER),
    }
}
