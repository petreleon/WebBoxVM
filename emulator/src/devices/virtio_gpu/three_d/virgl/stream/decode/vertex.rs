use super::super::super::{
    VIRGL_OBJECT_VERTEX_ELEMENTS, VertexBuffer, VertexElement, context::VertexLayout,
};

const CMD_CREATE_OBJECT: u8 = 1;
const CMD_BIND_OBJECT: u8 = 2;
const CMD_DESTROY_OBJECT: u8 = 3;
const CMD_SET_VERTEX_BUFFERS: u8 = 6;

#[derive(Clone, Copy)]
pub(in crate::devices::virtio_gpu::three_d::virgl::stream) enum Command {
    Create { handle: u32, layout: VertexLayout },
    Bind { handle: u32 },
    Destroy { handle: u32 },
    SetBuffer(Option<VertexBuffer>),
}

pub(super) fn decode(command: u8, object: u8, words: &[u32]) -> Option<Command> {
    match (command, object, words) {
        (CMD_CREATE_OBJECT, VIRGL_OBJECT_VERTEX_ELEMENTS, [handle, fields @ ..]) => layout(fields)
            .map(|layout| Command::Create {
                handle: *handle,
                layout,
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

fn layout(fields: &[u32]) -> Option<VertexLayout> {
    let element = |offset, divisor, buffer_index, format| VertexElement {
        offset,
        divisor,
        buffer_index,
        format,
    };
    match fields {
        [offset, divisor, index, format] => {
            VertexLayout::from_elements(&[element(*offset, *divisor, *index, *format)])
        }
        [o0, d0, i0, f0, o1, d1, i1, f1] => {
            VertexLayout::from_elements(&[element(*o0, *d0, *i0, *f0), element(*o1, *d1, *i1, *f1)])
        }
        [o0, d0, i0, f0, o1, d1, i1, f1, o2, d2, i2, f2] => VertexLayout::from_elements(&[
            element(*o0, *d0, *i0, *f0), element(*o1, *d1, *i1, *f1),
            element(*o2, *d2, *i2, *f2),
        ]),
        _ => None,
    }
}
