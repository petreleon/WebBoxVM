use super::super::super::shader::{MAX_SHADER_TEXT_BYTES, MAX_TGSI_TOKENS};
use super::super::super::{ShaderKind, VIRGL_OBJECT_SHADER};

const CMD_BIND_SHADER: u8 = 29;
const CMD_CREATE_OBJECT: u8 = 1;
const CMD_DESTROY_OBJECT: u8 = 3;
const OFFSET_CONTINUATION: u32 = 1 << 31;

#[derive(Clone)]
pub(in crate::devices::virtio_gpu::three_d::virgl::stream) enum Command {
    Create {
        handle: u32,
        kind: ShaderKind,
        token_count: u32,
        total_bytes: Option<u32>,
        offset: u32,
        chunk: Vec<u8>,
    },
    Bind {
        handle: Option<u32>,
        kind: ShaderKind,
    },
    Destroy {
        handle: u32,
    },
}

pub(super) fn decode(command: u8, object: u8, words: &[u32]) -> Option<Command> {
    match (command, object, words) {
        (CMD_BIND_SHADER, 0, [handle, kind]) => Some(Command::Bind {
            handle: (*handle != 0).then_some(*handle),
            kind: ShaderKind::from_pipe_type(*kind)?,
        }),
        (CMD_DESTROY_OBJECT, VIRGL_OBJECT_SHADER, [handle]) => {
            Some(Command::Destroy { handle: *handle })
        }
        (CMD_CREATE_OBJECT, VIRGL_OBJECT_SHADER, [handle, kind, offset, tokens, 0, text @ ..]) => {
            create(*handle, *kind, *offset, *tokens, text)
        }
        _ => None,
    }
}

fn create(handle: u32, kind: u32, offset: u32, tokens: u32, text: &[u32]) -> Option<Command> {
    if handle == 0 || tokens == 0 || tokens > MAX_TGSI_TOKENS || text.is_empty() {
        return None;
    }
    let continuation = offset & OFFSET_CONTINUATION != 0;
    let offset = offset & !OFFSET_CONTINUATION;
    let total_bytes = if continuation {
        if offset == 0 {
            return None;
        }
        None
    } else {
        let total = usize::try_from(offset).ok()?;
        let padded = total.div_ceil(4).checked_mul(4)?;
        if !(2..=MAX_SHADER_TEXT_BYTES).contains(&total) || text.len().checked_mul(4)? > padded {
            return None;
        }
        Some(offset)
    };
    Some(Command::Create {
        handle,
        kind: ShaderKind::from_pipe_type(kind)?,
        token_count: tokens,
        total_bytes,
        offset,
        chunk: text.iter().flat_map(|word| word.to_le_bytes()).collect(),
    })
}
