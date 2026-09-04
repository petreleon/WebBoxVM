use super::Command;
use super::super::super::{CopyRegion, VIRGL_CMD_CLEAR_SURFACE, VIRGL_OBJECT_SURFACE};
use crate::devices::virtio_gpu::protocol::Rect;

const CMD_CREATE_OBJECT: u8 = 1;
const CMD_DESTROY_OBJECT: u8 = 3;
const CMD_SET_FRAMEBUFFER_STATE: u8 = 5;
const CMD_CLEAR: u8 = 7;
const CMD_RESOURCE_COPY_REGION: u8 = 17;
const CLEAR_DEPTH: u32 = 1;
const CLEAR_COLOR0: u32 = 1 << 2;

pub(super) fn decode(command: u8, object: u8, words: &[u32]) -> Option<Command> {
    match (command, object, words) {
        (CMD_CREATE_OBJECT, VIRGL_OBJECT_SURFACE, [handle, resource, format, level, layers]) => {
            Some(Command::CreateSurface { handle: *handle, resource: *resource, format: *format, level: *level, layers: *layers })
        }
        (CMD_DESTROY_OBJECT, VIRGL_OBJECT_SURFACE, [handle]) => Some(Command::DestroySurface { handle: *handle }),
        (CMD_SET_FRAMEBUFFER_STATE, 0, [0, 0]) => Some(Command::SetFramebuffer { color: None, depth: None }),
        (CMD_SET_FRAMEBUFFER_STATE, 0, [1, 0, color]) if *color != 0 => {
            Some(Command::SetFramebuffer { color: Some(*color), depth: None })
        }
        (CMD_SET_FRAMEBUFFER_STATE, 0, [1, depth, color]) if *depth != 0 && *color != 0 => {
            Some(Command::SetFramebuffer { color: Some(*color), depth: Some(*depth) })
        }
        (CMD_CLEAR, 0, [buffers, red, green, blue, alpha, 0, 0, 0]) if *buffers == CLEAR_COLOR0 => {
            Some(Command::Clear { color: color(*red, *green, *blue, *alpha), depth: false })
        }
        (CMD_CLEAR, 0, [buffers, red, green, blue, alpha, depth, 0, 0])
            if *buffers == (CLEAR_COLOR0 | CLEAR_DEPTH) && *depth == 1.0f32.to_bits() => {
            Some(Command::Clear { color: color(*red, *green, *blue, *alpha), depth: true })
        }
        (VIRGL_CMD_CLEAR_SURFACE, 0, [state, handle, red, green, blue, alpha, x, y, width, height])
            if *state == CLEAR_COLOR0 << 1 => Some(Command::ClearSurface {
                handle: *handle, color: color(*red, *green, *blue, *alpha),
                rect: Rect { x: *x, y: *y, width: *width, height: *height },
            }),
        (CMD_RESOURCE_COPY_REGION, 0, [dst_resource, dst_level, dst_x, dst_y, dst_z, src_resource,
            src_level, src_x, src_y, src_z, width, height, depth]) => Some(Command::CopyRegion(CopyRegion {
                dst_resource: *dst_resource, dst_level: *dst_level, dst_x: *dst_x, dst_y: *dst_y,
                dst_z: *dst_z, src_resource: *src_resource, src_level: *src_level,
                src_rect: Rect { x: *src_x, y: *src_y, width: *width, height: *height },
                src_z: *src_z, depth: *depth,
            })),
        _ => None,
    }
}

fn color(red: u32, green: u32, blue: u32, alpha: u32) -> [f32; 4] {
    [red, green, blue, alpha].map(f32::from_bits)
}
