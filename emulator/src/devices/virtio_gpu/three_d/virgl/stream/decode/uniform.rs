use super::super::super::ShaderKind;

const CMD_SET_UNIFORM_BUFFER: u8 = 27;
const UNIFORM_BYTES: u32 = 16;

#[derive(Clone, Copy)]
pub(in crate::devices::virtio_gpu::three_d::virgl::stream) enum Command {
    Clear(ShaderKind),
    Set {
        kind: ShaderKind,
        resource: u32,
        offset: u32,
    },
}

pub(super) fn decode(command: u8, object: u8, words: &[u32]) -> Option<Command> {
    match (command, object, words) {
        (CMD_SET_UNIFORM_BUFFER, 0, [kind, 0, 0, 0, 0]) => {
            Some(Command::Clear(ShaderKind::from_pipe_type(*kind)?))
        }
        (CMD_SET_UNIFORM_BUFFER, 0, [kind, 0, offset, UNIFORM_BYTES, resource])
            if *resource != 0 && offset.is_multiple_of(4) =>
        {
            Some(Command::Set {
                kind: ShaderKind::from_pipe_type(*kind)?,
                resource: *resource,
                offset: *offset,
            })
        }
        _ => None,
    }
}
