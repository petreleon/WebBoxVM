mod fragment;
mod shape;
mod vertex;

use super::{MAX_SHADER_TEXT_BYTES, Shader, ShaderKind};

pub(in crate::devices::virtio_gpu::three_d::virgl) fn parse(
    kind: ShaderKind,
    source: &[u8],
) -> Option<Shader> {
    let lines = lines(source)?;
    let program = match kind {
        ShaderKind::Vertex => vertex::parse(&lines),
        ShaderKind::Fragment => fragment::parse(&lines),
    }?;
    Some(Shader { kind, program })
}

fn lines(source: &[u8]) -> Option<Vec<&str>> {
    if source.len() < 2 || source.len() > MAX_SHADER_TEXT_BYTES || *source.last()? != 0 {
        return None;
    }
    let text = std::str::from_utf8(&source[..source.len() - 1]).ok()?;
    if text
        .bytes()
        .any(|byte| !matches!(byte, b'\t' | b'\n' | b'\r' | 0x20..=0x7e))
    {
        return None;
    }
    Some(
        text.lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(strip_instruction_id)
            .collect(),
    )
}

fn strip_instruction_id(line: &str) -> &str {
    let Some((prefix, rest)) = line.split_once(':') else {
        return line;
    };
    (!prefix.is_empty() && prefix.bytes().all(|byte| byte.is_ascii_digit()))
        .then(|| rest.trim())
        .unwrap_or(line)
}
