use super::super::super::{DepthCompare, VIRGL_OBJECT_DSA};

const CMD_BIND_OBJECT: u8 = 2;
const CMD_CREATE_OBJECT: u8 = 1;
const CMD_DESTROY_OBJECT: u8 = 3;
const DSA_DEPTH_REQUIRED_BITS: u32 = 3;

#[derive(Clone, Copy)]
pub(in crate::devices::virtio_gpu::three_d::virgl::stream) enum Command {
    Create { handle: u32, compare: DepthCompare },
    Bind { handle: Option<u32> },
    Destroy { handle: u32 },
}

pub(super) fn decode(command: u8, object: u8, words: &[u32]) -> Option<Command> {
    match (command, object, words) {
        (CMD_CREATE_OBJECT, VIRGL_OBJECT_DSA, [handle, state, 0, 0, 0])
            if *handle != 0
                && (*state & DSA_DEPTH_REQUIRED_BITS) == DSA_DEPTH_REQUIRED_BITS
                && (*state & !31) == 0 =>
            DepthCompare::from_wire(*state >> 2).map(|compare| Command::Create { handle: *handle, compare }),
        (CMD_BIND_OBJECT, VIRGL_OBJECT_DSA, [handle]) => Some(Command::Bind {
            handle: (*handle != 0).then_some(*handle),
        }),
        (CMD_DESTROY_OBJECT, VIRGL_OBJECT_DSA, [handle]) if *handle != 0 => {
            Some(Command::Destroy { handle: *handle })
        }
        _ => None,
    }
}
