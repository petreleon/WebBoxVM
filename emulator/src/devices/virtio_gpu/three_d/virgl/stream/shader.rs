use super::super::VirglContext;
use super::super::shader::parse;
use super::decode::shader::Command;
use crate::devices::virtio_gpu::protocol::RESP_ERR_INVALID_PARAMETER;

pub(super) fn apply(context: &mut VirglContext, command: Command) -> Result<(), u32> {
    match command {
        Command::Create {
            handle,
            kind,
            source,
        } => context
            .create_shader(
                handle,
                parse(kind, &source).ok_or(RESP_ERR_INVALID_PARAMETER)?,
            )
            .then_some(())
            .ok_or(RESP_ERR_INVALID_PARAMETER),
        Command::Bind { handle, kind } => context
            .bind_shader(kind, handle)
            .then_some(())
            .ok_or(RESP_ERR_INVALID_PARAMETER),
        Command::Destroy { handle } => context
            .destroy_shader(handle)
            .then_some(())
            .ok_or(RESP_ERR_INVALID_PARAMETER),
    }
}
