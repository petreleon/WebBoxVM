use super::super::super::ShaderKind;

const CMD_SET_UNIFORM_BUFFER: u8 = 27;
const PIPE_SHADER_VERTEX: u32 = 0;
const PIPE_SHADER_FRAGMENT: u32 = 1;
const OFFSET_BYTES: u32 = 16;
const MATRIX_BYTES: u32 = 64;

#[derive(Clone, Copy)]
pub(in crate::devices::virtio_gpu::three_d::virgl::stream) enum Command {
    Clear(ShaderKind),
    SetVertexOffset {
        resource: u32,
        offset: u32,
    },
    SetVertexMatrix {
        resource: u32,
        offset: u32,
    },
    SetFragment {
        resource: u32,
        offset: u32,
    },
}

pub(super) fn decode(command: u8, object: u8, words: &[u32]) -> Option<Command> {
    match (command, object, words) {
        (CMD_SET_UNIFORM_BUFFER, 0, [kind, 0, 0, 0, 0]) => {
            Some(Command::Clear(ShaderKind::from_pipe_type(*kind)?))
        }
        (CMD_SET_UNIFORM_BUFFER, 0, [PIPE_SHADER_VERTEX, 0, offset, OFFSET_BYTES, resource])
            if *resource != 0 && offset.is_multiple_of(4) =>
        {
            Some(Command::SetVertexOffset { resource: *resource, offset: *offset })
        }
        (CMD_SET_UNIFORM_BUFFER, 0, [PIPE_SHADER_VERTEX, 0, offset, MATRIX_BYTES, resource])
            if *resource != 0 && offset.is_multiple_of(4) =>
        {
            Some(Command::SetVertexMatrix { resource: *resource, offset: *offset })
        }
        (CMD_SET_UNIFORM_BUFFER, 0, [PIPE_SHADER_FRAGMENT, 0, offset, OFFSET_BYTES, resource])
            if *resource != 0 && offset.is_multiple_of(4) =>
        {
            Some(Command::SetFragment { resource: *resource, offset: *offset })
        }
        _ => None,
    }
}
