use super::super::super::VIRGL_OBJECT_BLEND;

const CMD_BIND_OBJECT: u8 = 2;
const CMD_CREATE_OBJECT: u8 = 1;
const CMD_DESTROY_OBJECT: u8 = 3;
const SOURCE_OVER: u32 = 1 | (3 << 4) | (19 << 9) | (1 << 17) | (19 << 22) | (15 << 27);

#[derive(Clone, Copy)]
pub(in crate::devices::virtio_gpu::three_d::virgl::stream) enum Command {
    Create { handle: u32 },
    Bind { handle: Option<u32> },
    Destroy { handle: u32 },
}

pub(super) fn decode(command: u8, object: u8, words: &[u32]) -> Option<Command> {
    match (command, object, words) {
        (
            CMD_CREATE_OBJECT,
            VIRGL_OBJECT_BLEND,
            [handle, 0, 0, SOURCE_OVER, 0, 0, 0, 0, 0, 0, 0],
        ) => Some(Command::Create { handle: *handle }),
        (CMD_BIND_OBJECT, VIRGL_OBJECT_BLEND, [handle]) => Some(Command::Bind {
            handle: (*handle != 0).then_some(*handle),
        }),
        (CMD_DESTROY_OBJECT, VIRGL_OBJECT_BLEND, [handle]) => {
            Some(Command::Destroy { handle: *handle })
        }
        _ => None,
    }
}
