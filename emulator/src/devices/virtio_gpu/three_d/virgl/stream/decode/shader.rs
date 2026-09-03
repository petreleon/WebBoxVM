use super::super::super::{ShaderKind, VIRGL_OBJECT_SHADER};

const CMD_BIND_SHADER: u8 = 29;
const CMD_CREATE_OBJECT: u8 = 1;
const CMD_DESTROY_OBJECT: u8 = 3;
const MAX_TGSI_TOKENS: u32 = 256;

#[derive(Clone)]
pub(in crate::devices::virtio_gpu::three_d::virgl::stream) enum Command {
    Create {
        handle: u32,
        kind: ShaderKind,
        source: Vec<u8>,
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
    let source_len = usize::try_from(offset).ok()?;
    if handle == 0 || offset & (1 << 31) != 0 || tokens == 0 || tokens > MAX_TGSI_TOKENS {
        return None;
    }
    let bytes: Vec<u8> = text.iter().flat_map(|word| word.to_le_bytes()).collect();
    if source_len > bytes.len()
        || source_len < 2
        || bytes.get(source_len - 1) != Some(&0)
        || bytes[..source_len - 1].contains(&0)
    {
        return None;
    }
    Some(Command::Create {
        handle,
        kind: ShaderKind::from_pipe_type(kind)?,
        source: bytes[..source_len].to_vec(),
    })
}
