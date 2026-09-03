use super::super::super::IndexBuffer;

const CMD_SET_INDEX_BUFFER: u8 = 11;

#[derive(Clone, Copy)]
pub(in crate::devices::virtio_gpu::three_d::virgl::stream) enum Command {
    SetBuffer(Option<IndexBuffer>),
}

pub(super) fn decode(command: u8, object: u8, words: &[u32]) -> Option<Command> {
    match (command, object, words) {
        (CMD_SET_INDEX_BUFFER, 0, [0]) => Some(Command::SetBuffer(None)),
        (CMD_SET_INDEX_BUFFER, 0, [resource, index_size, offset]) if *resource != 0 => {
            Some(Command::SetBuffer(Some(IndexBuffer {
                resource: *resource,
                index_size: *index_size,
                offset: *offset,
            })))
        }
        _ => None,
    }
}
