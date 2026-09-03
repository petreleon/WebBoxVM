use super::{MAX_SHADER_TEXT_BYTES, Shader, ShaderKind, ShaderProgram};

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

fn vertex(lines: &[&str]) -> Option<ShaderProgram> {
    match lines {
        [
            "VERT",
            "DCL IN[0]",
            "DCL OUT[0], POSITION",
            "MOV OUT[0], IN[0]",
            "END",
        ] => Some(ShaderProgram::VertexPassthrough),
        [
            "VERT",
            "DCL IN[0..1]",
            "DCL OUT[0], POSITION",
            "DCL OUT[1], GENERIC[0]",
            "MOV OUT[0], IN[0]",
            "MOV OUT[1], IN[1]",
            "END",
        ] => Some(ShaderProgram::VertexGeneric),
        _ => None,
    }
}

fn fragment(lines: &[&str]) -> Option<ShaderProgram> {
    if lines
        == [
            "FRAG",
            "DCL IN[0], GENERIC[0], LINEAR",
            "DCL OUT[0], COLOR[0]",
            "MOV OUT[0], IN[0]",
            "END",
        ]
    {
        return Some(ShaderProgram::FragmentVertexColor);
    }
    if lines
        == [
            "FRAG",
            "DCL IN[0], GENERIC[0], LINEAR",
            "DCL SAMP[0]",
            "DCL SVIEW[0], 2D, FLOAT",
            "DCL OUT[0], COLOR[0]",
            "DCL TEMP[0]",
            "TEX TEMP[0], IN[0], SAMP[0], 2D",
            "MOV OUT[0], TEMP[0]",
            "END",
        ]
    {
        return Some(ShaderProgram::FragmentTextured);
    }
    if lines
        == [
            "FRAG",
            "DCL IN[0], GENERIC[0], LINEAR",
            "DCL SAMP[0..1]",
            "DCL SVIEW[0..1], 2D, FLOAT",
            "DCL OUT[0], COLOR[0]",
            "DCL TEMP[0..1]",
            "TEX TEMP[0], IN[0], SAMP[0], 2D",
            "TEX TEMP[1], IN[0], SAMP[1], 2D",
            "MUL OUT[0], TEMP[0], TEMP[1]",
            "END",
        ]
    {
        return Some(ShaderProgram::FragmentTexturedMultiply);
    }
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
