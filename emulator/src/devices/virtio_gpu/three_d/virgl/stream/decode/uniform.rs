const CMD_SET_UNIFORM_BUFFER: u8 = 27;
const PIPE_SHADER_FRAGMENT: u32 = 1;
const UNIFORM_BYTES: u32 = 16;

#[derive(Clone, Copy)]
pub(in crate::devices::virtio_gpu::three_d::virgl::stream) enum Command {
    Clear,
    SetFragment { resource: u32, offset: u32 },
}

pub(super) fn decode(command: u8, object: u8, words: &[u32]) -> Option<Command> {
    match (command, object, words) {
        (CMD_SET_UNIFORM_BUFFER, 0, [PIPE_SHADER_FRAGMENT, 0, 0, 0, 0]) => Some(Command::Clear),
        (CMD_SET_UNIFORM_BUFFER, 0, [PIPE_SHADER_FRAGMENT, 0, offset, UNIFORM_BYTES, resource])
            if *resource != 0 && offset.is_multiple_of(4) =>
        {
            Some(Command::SetFragment {
                resource: *resource,
                offset: *offset,
            })
        }
        _ => None,
    }
}
