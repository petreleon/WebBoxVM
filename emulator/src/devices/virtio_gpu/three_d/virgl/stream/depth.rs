use super::super::VirglContext;
use super::decode::depth::Command;
use crate::devices::virtio_gpu::protocol::RESP_ERR_INVALID_PARAMETER;

pub(super) fn apply(context: &mut VirglContext, command: Command) -> Result<(), u32> {
    let valid = match command {
        Command::Create { handle, compare } => context.create_depth_state(handle, compare),
        Command::Bind { handle } => context.bind_depth_state(handle),
        Command::Destroy { handle } => context.destroy_depth_state(handle),
    };
    valid.then_some(()).ok_or(RESP_ERR_INVALID_PARAMETER)
}
