pub(super) mod blend;
pub(super) mod constant;
mod core;
pub(super) mod depth;
pub(super) mod draw;
pub(super) mod index;
pub(super) mod inline;
pub(super) mod sampler;
pub(super) mod shader;
pub(super) mod state;
pub(super) mod uniform;
pub(super) mod vertex;
use super::super::draw::DrawCall;
use super::super::inline::InlineWrite;
use super::super::{CopyRegion, MAX_VIRGL_SUBMIT_BYTES};
use crate::devices::virtio_gpu::protocol::{Rect, read_u32};
const CMD_NOP: u8 = 0;

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
    DestroySurface { handle: u32 },
    SetFramebuffer { color: Option<u32>, depth: Option<u32> },
    Clear { color: [f32; 4], depth: bool },
    ClearSurface {
        handle: u32,
        color: [f32; 4],
        rect: Rect,
    },
    CopyRegion(CopyRegion),
    InlineWrite(InlineWrite),
    Draw(DrawCall),
    Blend(blend::Command),
    Constant(constant::Command),
    Depth(depth::Command),
    Uniform(uniform::Command),
    Vertex(vertex::Command),
    Index(index::Command),
    Sampler(sampler::Command),
    Shader(shader::Command),
    State(state::Command),
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
    let (command, object) = (header as u8, (header >> 8) as u8);
    if command == CMD_NOP && object == 0 && words.is_empty() { return Some(Command::Nop); }
    core::decode(command, object, words)
        .or_else(|| vertex::decode(command, object, words).map(Command::Vertex))
        .or_else(|| constant::decode(command, object, words).map(Command::Constant))
        .or_else(|| depth::decode(command, object, words).map(Command::Depth))
        .or_else(|| uniform::decode(command, object, words).map(Command::Uniform))
        .or_else(|| inline::decode(command, object, words).map(Command::InlineWrite))
        .or_else(|| index::decode(command, object, words).map(Command::Index))
        .or_else(|| sampler::decode(command, object, words).map(Command::Sampler))
        .or_else(|| draw::decode(command, object, words).map(Command::Draw))
        .or_else(|| blend::decode(command, object, words).map(Command::Blend))
        .or_else(|| shader::decode(command, object, words).map(Command::Shader))
        .or_else(|| state::decode(command, object, words).map(Command::State))
}
