use super::{Shader, ShaderKind, ShaderProgram};

const MAX_SHADER_SOURCE_BYTES: usize = 4 * 1024;

pub(in crate::devices::virtio_gpu::three_d::virgl) fn parse(
    kind: ShaderKind,
    source: &[u8],
) -> Option<Shader> {
    let lines = lines(source)?;
    let program = match kind {
        ShaderKind::Vertex => vertex(&lines),
        ShaderKind::Fragment => fragment(&lines),
    }?;
    Some(Shader { kind, program })
}

fn lines(source: &[u8]) -> Option<Vec<&str>> {
    if source.len() < 2 || source.len() > MAX_SHADER_SOURCE_BYTES || *source.last()? != 0 {
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

fn vertex(lines: &[&str]) -> Option<ShaderProgram> {
    (lines
        == [
            "VERT",
            "DCL IN[0]",
            "DCL OUT[0], POSITION",
            "MOV OUT[0], IN[0]",
            "END",
        ])
    .then_some(ShaderProgram::VertexPassthrough)
}

fn fragment(lines: &[&str]) -> Option<ShaderProgram> {
    if lines.len() != 5
        || lines[0] != "FRAG"
        || lines[1] != "DCL OUT[0], COLOR"
        || lines[3] != "MOV OUT[0], IMM[0]"
        || lines[4] != "END"
    {
        return None;
    }
    color(lines[2]).map(ShaderProgram::FragmentSolid)
}

fn color(line: &str) -> Option<[u32; 4]> {
    let body = line.strip_prefix("IMM[0] FLT32 {")?.strip_suffix('}')?;
    let values: Vec<_> = body.split(',').map(str::trim).collect();
    (values.len() == 4)
        .then(|| {
            values
                .into_iter()
                .map(float_bits)
                .collect::<Option<Vec<_>>>()?
                .try_into()
                .ok()
        })
        .flatten()
}

fn float_bits(value: &str) -> Option<u32> {
    let bits = value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"))
        .map(|hex| u32::from_str_radix(hex, 16).ok())
        .unwrap_or_else(|| value.parse::<f32>().ok().map(f32::to_bits))?;
    let value = f32::from_bits(bits);
    (value.is_finite() && (0.0..=1.0).contains(&value)).then_some(bits)
}
