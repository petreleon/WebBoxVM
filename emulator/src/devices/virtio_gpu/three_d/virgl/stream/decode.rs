pub(super) mod draw;
pub(super) mod shader;
pub(super) mod vertex;

use super::super::draw::DrawCall;
use super::super::{
    CopyRegion, MAX_VIRGL_SUBMIT_BYTES, VIRGL_CMD_CLEAR_SURFACE, VIRGL_OBJECT_SURFACE,
};
use crate::devices::virtio_gpu::protocol::{Rect, read_u32};

const CMD_NOP: u8 = 0;
const CMD_CREATE_OBJECT: u8 = 1;
const CMD_DESTROY_OBJECT: u8 = 3;
const CMD_SET_FRAMEBUFFER_STATE: u8 = 5;
const CMD_CLEAR: u8 = 7;
const CMD_RESOURCE_COPY_REGION: u8 = 17;
const CLEAR_COLOR0: u32 = 1 << 2;

#[derive(Clone)]
pub(super) enum Command {
    Nop,
    CreateSurface {
        handle: u32,
        resource: u32,
        format: u32,
        level: u32,
        layers: u32,
    },
    DestroySurface {
        handle: u32,
    },
    SetFramebuffer {
        surface: Option<u32>,
    },
    Clear {
        color: [f32; 4],
    },
    ClearSurface {
        handle: u32,
        color: [f32; 4],
        rect: Rect,
    },
    CopyRegion(CopyRegion),
    Draw(DrawCall),
    Vertex(vertex::Command),
    Shader(shader::Command),
}

pub(super) fn decode_stream(input: &[u8]) -> Option<Vec<Command>> {
    let size = usize::try_from(read_u32(input, 24)?).ok()?;
    if input.len() != size.checked_add(32)?
        || read_u32(input, 28)? != 0
        || size == 0
        || size > MAX_VIRGL_SUBMIT_BYTES
        || size % 4 != 0
    {
        return None;
    }
    let words: Vec<u32> = (0..size / 4)
        .map(|index| read_u32(input, 32 + index * 4))
        .collect::<Option<_>>()?;
    let mut offset = 0;
    let mut commands = Vec::new();
    while offset < words.len() {
        let header = words[offset];
        let length = usize::try_from(header >> 16).ok()?;
        let end = offset.checked_add(length.checked_add(1)?)?;
        if end > words.len() {
            return None;
        }
        commands.push(decode_command(header, &words[offset + 1..end])?);
        offset = end;
    }
    Some(commands)
}

fn decode_command(header: u32, words: &[u32]) -> Option<Command> {
    let command = header as u8;
    let object = (header >> 8) as u8;
    match (command, object, words) {
        (CMD_NOP, 0, []) => Some(Command::Nop),
        (CMD_CREATE_OBJECT, VIRGL_OBJECT_SURFACE, [handle, resource, format, level, layers]) => {
            Some(Command::CreateSurface {
                handle: *handle,
                resource: *resource,
                format: *format,
                level: *level,
                layers: *layers,
            })
        }
        (CMD_DESTROY_OBJECT, VIRGL_OBJECT_SURFACE, [handle]) => {
            Some(Command::DestroySurface { handle: *handle })
        }
        (CMD_SET_FRAMEBUFFER_STATE, 0, [0, 0]) => Some(Command::SetFramebuffer { surface: None }),
        (CMD_SET_FRAMEBUFFER_STATE, 0, [1, 0, surface]) if *surface != 0 => {
            Some(Command::SetFramebuffer {
                surface: Some(*surface),
            })
        }
        (CMD_CLEAR, 0, [buffers, red, green, blue, alpha, _, _, _]) if *buffers == CLEAR_COLOR0 => {
            Some(Command::Clear {
                color: [
                    f32::from_bits(*red),
                    f32::from_bits(*green),
                    f32::from_bits(*blue),
                    f32::from_bits(*alpha),
                ],
            })
        }
        (
            VIRGL_CMD_CLEAR_SURFACE,
            0,
            [state, handle, red, green, blue, alpha, x, y, width, height],
        ) if *state == CLEAR_COLOR0 << 1 => Some(Command::ClearSurface {
            handle: *handle,
            color: [
                f32::from_bits(*red),
                f32::from_bits(*green),
                f32::from_bits(*blue),
                f32::from_bits(*alpha),
            ],
            rect: Rect {
                x: *x,
                y: *y,
                width: *width,
                height: *height,
            },
        }),
        (
            CMD_RESOURCE_COPY_REGION,
            0,
            [
                dst_resource,
                dst_level,
                dst_x,
                dst_y,
                dst_z,
                src_resource,
                src_level,
                src_x,
                src_y,
                src_z,
                width,
                height,
                depth,
            ],
        ) => Some(Command::CopyRegion(CopyRegion {
            dst_resource: *dst_resource,
            dst_level: *dst_level,
            dst_x: *dst_x,
            dst_y: *dst_y,
            dst_z: *dst_z,
            src_resource: *src_resource,
            src_level: *src_level,
            src_rect: Rect {
                x: *src_x,
                y: *src_y,
                width: *width,
                height: *height,
            },
            src_z: *src_z,
            depth: *depth,
        })),
        _ => vertex::decode(command, object, words)
            .map(Command::Vertex)
            .or_else(|| draw::decode(command, object, words).map(Command::Draw))
            .or_else(|| shader::decode(command, object, words).map(Command::Shader)),
    }
}
