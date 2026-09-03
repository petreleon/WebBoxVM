use super::super::super::{VIRGL_OBJECT_VERTEX_ELEMENTS, VertexBuffer, VertexElement};

const CMD_CREATE_OBJECT: u8 = 1;
const CMD_BIND_OBJECT: u8 = 2;
const CMD_DESTROY_OBJECT: u8 = 3;
const CMD_SET_VERTEX_BUFFERS: u8 = 6;

#[derive(Clone, Copy)]
pub(in crate::devices::virtio_gpu::three_d::virgl::stream) enum Command {
    Create { handle: u32, element: VertexElement },
    Bind { handle: u32 },
    Destroy { handle: u32 },
    SetBuffer(Option<VertexBuffer>),
}

pub(super) fn decode(command: u8, object: u8, words: &[u32]) -> Option<Command> {
    match (command, object, words) {
        (
            CMD_CREATE_OBJECT,
            VIRGL_OBJECT_VERTEX_ELEMENTS,
            [handle, offset, divisor, index, format],
        ) => Some(Command::Create {
            handle: *handle,
            element: VertexElement {
                offset: *offset,
                divisor: *divisor,
                buffer_index: *index,
                format: *format,
            },
        }),
        (CMD_BIND_OBJECT, VIRGL_OBJECT_VERTEX_ELEMENTS, [handle]) => {
            Some(Command::Bind { handle: *handle })
        }
        (CMD_DESTROY_OBJECT, VIRGL_OBJECT_VERTEX_ELEMENTS, [handle]) => {
            Some(Command::Destroy { handle: *handle })
        }
        (CMD_SET_VERTEX_BUFFERS, 0, []) => Some(Command::SetBuffer(None)),
        (CMD_SET_VERTEX_BUFFERS, 0, [stride, offset, resource]) => {
            Some(Command::SetBuffer(Some(VertexBuffer {
                stride: *stride,
                offset: *offset,
                resource: *resource,
            })))
        }
        _ => None,
    }
}
