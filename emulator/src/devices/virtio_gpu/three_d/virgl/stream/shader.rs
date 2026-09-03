use super::super::VirglContext;
use super::super::shader::parse;
use super::decode::shader::Command;
use crate::devices::virtio_gpu::protocol::RESP_ERR_INVALID_PARAMETER;

pub(super) fn apply(context: &mut VirglContext, command: Command) -> Result<(), u32> {
    match command {
        Command::Create {
            handle,
            kind,
            token_count,
            total_bytes,
            offset,
            chunk,
        } => {
            let Some(source) = context
                .accept_shader_chunk(handle, kind, token_count, total_bytes, offset, chunk)
                .ok_or(RESP_ERR_INVALID_PARAMETER)?
            else {
                return Ok(());
            };
            let shader = parse(kind, &source).ok_or(RESP_ERR_INVALID_PARAMETER)?;
            if token_count.saturating_add(10) < shader.tgsi_token_count() {
                return Err(RESP_ERR_INVALID_PARAMETER);
            }
            context
                .create_shader(handle, shader)
                .then_some(())
                .ok_or(RESP_ERR_INVALID_PARAMETER)
        }
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
