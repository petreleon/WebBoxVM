use super::super::super::VIRGL_OBJECT_RASTERIZER;
use super::super::super::context::Viewport;
use crate::devices::virtio_gpu::protocol::Rect;

const CMD_BIND_OBJECT: u8 = 2;
const CMD_CREATE_OBJECT: u8 = 1;
const CMD_DESTROY_OBJECT: u8 = 3;
const CMD_SET_SCISSOR_STATE: u8 = 15;
const CMD_SET_VIEWPORT_STATE: u8 = 4;
const RASTERIZER_BASE: u32 = (1 << 1) | (1 << 29) | (1 << 30);
const RASTERIZER_SCISSOR: u32 = RASTERIZER_BASE | (1 << 14);

#[derive(Clone, Copy)]
pub(in crate::devices::virtio_gpu::three_d::virgl::stream) enum Command {
    CreateRasterizer { handle: u32, scissor: bool },
    BindRasterizer { handle: Option<u32> },
    DestroyRasterizer { handle: u32 },
    Viewport(Viewport),
    Scissor(Rect),
}

pub(super) fn decode(command: u8, object: u8, words: &[u32]) -> Option<Command> {
    match (command, object, words) {
        (
            CMD_CREATE_OBJECT,
            VIRGL_OBJECT_RASTERIZER,
            [handle, state, point, 0, 0, line, 0, 0, 0],
        ) => rasterizer(*handle, *state, *point, *line),
        (CMD_BIND_OBJECT, VIRGL_OBJECT_RASTERIZER, [handle]) => Some(Command::BindRasterizer {
            handle: (*handle != 0).then_some(*handle),
        }),
        (CMD_DESTROY_OBJECT, VIRGL_OBJECT_RASTERIZER, [handle]) => {
            Some(Command::DestroyRasterizer { handle: *handle })
        }
        (CMD_SET_VIEWPORT_STATE, 0, [0, x, y, z, tx, ty, tz]) => Viewport::new(
            [f32::from_bits(*x), f32::from_bits(*y), f32::from_bits(*z)],
            [
                f32::from_bits(*tx),
                f32::from_bits(*ty),
                f32::from_bits(*tz),
            ],
        )
        .map(Command::Viewport),
        (CMD_SET_SCISSOR_STATE, 0, [0, min, max]) => scissor(*min, *max).map(Command::Scissor),
        _ => None,
    }
}

fn rasterizer(handle: u32, state: u32, point: u32, line: u32) -> Option<Command> {
    let scissor = match state {
        RASTERIZER_BASE => false,
        RASTERIZER_SCISSOR => true,
        _ => return None,
    };
    (handle != 0 && point == 1.0f32.to_bits() && line == 1.0f32.to_bits())
        .then_some(Command::CreateRasterizer { handle, scissor })
}

fn scissor(min: u32, max: u32) -> Option<Rect> {
    let (min_x, min_y) = (min & 0xffff, min >> 16);
    let (max_x, max_y) = (max & 0xffff, max >> 16);
    let width = max_x.checked_sub(min_x)?;
    let height = max_y.checked_sub(min_y)?;
    (width != 0 && height != 0).then_some(Rect {
        x: min_x,
        y: min_y,
        width,
        height,
    })
}
