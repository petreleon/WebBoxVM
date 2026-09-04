use super::shape::{self, Operation};
use super::super::ShaderProgram;

pub(super) fn parse(lines: &[&str]) -> Option<ShaderProgram> {
    let shape = shape::parse(lines, "FRAG")?;
    match shape.operations() {
        [Operation::Mov(output, source), Operation::End]
            if color_output(&shape, output) && shape.has_register("CONST[0][0]") && shape::same(source, "CONST[0][0]") =>
        {
            Some(ShaderProgram::FragmentConstant)
        }
        [Operation::Mov(output, source), Operation::End]
            if generic_input(&shape, 0) && color_output(&shape, output) && shape::same(source, "IN[0]") =>
        {
            Some(ShaderProgram::FragmentVertexColor)
        }
        [Operation::Mul(output, left, right), Operation::End]
            if generic_input(&shape, 0) && shape.has_register("CONST[0][0]") && color_output(&shape, output)
                && product(left, right, "IN[0]", "CONST[0][0]") =>
        {
            Some(ShaderProgram::FragmentVertexColorConstant)
        }
        [Operation::Tex(temp, coordinates, sampler, target), Operation::Mov(output, source), Operation::End]
            if texture(&shape, 0) && shape::same(temp, "TEMP[0]") && shape::same(coordinates, "IN[0]")
                && shape::same(sampler, "SAMP[0]") && target == &"2D" && color_output(&shape, output) && shape::same(source, "TEMP[0]") =>
        {
            Some(ShaderProgram::FragmentTextured)
        }
        [Operation::Tex(left, coordinates0, sampler0, target0), Operation::Tex(right, coordinates1, sampler1, target1), Operation::Mul(output, source0, source1), Operation::End]
            if texture_pair(&shape) && shape::same(left, "TEMP[0]") && shape::same(coordinates0, "IN[0]")
                && shape::same(sampler0, "SAMP[0]") && target0 == &"2D" && shape::same(right, "TEMP[1]")
                && shape::same(coordinates1, "IN[0]") && shape::same(sampler1, "SAMP[1]") && target1 == &"2D"
                && color_output(&shape, output) && shape::same(source0, "TEMP[0]") && shape::same(source1, "TEMP[1]") =>
        {
            Some(ShaderProgram::FragmentTexturedMultiply)
        }
        [Operation::Tex(temp, coordinates, sampler, target), Operation::Mul(output, sampled, color), Operation::End]
            if texture_color(&shape) && shape::same(temp, "TEMP[0]") && shape::same(coordinates, "IN[1]")
                && shape::same(sampler, "SAMP[0]") && target == &"2D" && color_output(&shape, output)
                && shape::same(sampled, "TEMP[0]") && shape::same(color, "IN[0]") =>
        {
            Some(ShaderProgram::FragmentTexturedVertexColor)
        }
        [Operation::Mov(output, source), Operation::End] => solid(&shape, output, source).map(ShaderProgram::FragmentSolid),
        _ => None,
    }
}

fn generic_input(shape: &shape::Shape<'_>, index: u32) -> bool {
    shape.has_register(&format!("IN[{index}]")) && shape.has_semantic(&format!("IN[{index}]"), &format!("GENERIC[{index}]"))
}

fn color_output(shape: &shape::Shape<'_>, output: &str) -> bool {
    shape::same(output, "OUT[0]")
        && (shape.has_semantic("OUT[0]", "COLOR") || shape.has_semantic("OUT[0]", "COLOR[0]"))
}

fn texture(shape: &shape::Shape<'_>, sampler: u32) -> bool {
    generic_input(shape, 0) && shape.has_register(&format!("TEMP[{sampler}]"))
        && shape.has_register(&format!("SAMP[{sampler}]")) && shape.has_semantic(&format!("SVIEW[{sampler}]"), "2D")
}

fn product(left: &str, right: &str, first: &str, second: &str) -> bool {
    (shape::same(left, first) && shape::same(right, second))
        || (shape::same(left, second) && shape::same(right, first))
}

fn texture_pair(shape: &shape::Shape<'_>) -> bool {
    texture(shape, 0) && texture(shape, 1)
}

fn texture_color(shape: &shape::Shape<'_>) -> bool {
    texture(shape, 0) && generic_input(shape, 1)
}

fn solid(shape: &shape::Shape<'_>, output: &str, source: &str) -> Option<[u32; 4]> {
    (color_output(shape, output) && shape::same(source, "IMM[0]")).then(|| color(shape.immediate("IMM[0]")?)).flatten()
}

fn color(line: &str) -> Option<[u32; 4]> {
    let body = line.strip_prefix("IMM[0] FLT32 {")?.strip_suffix('}')?;
    let values: Vec<_> = body.split(',').map(str::trim).collect();
    (values.len() == 4).then(|| values.into_iter().map(float_bits).collect::<Option<Vec<_>>>()?.try_into().ok()).flatten()
}

fn float_bits(value: &str) -> Option<u32> {
    let bits = value.strip_prefix("0x").or_else(|| value.strip_prefix("0X"))
        .map(|hex| u32::from_str_radix(hex, 16).ok())
        .unwrap_or_else(|| value.parse::<f32>().ok().map(f32::to_bits))?;
    let value = f32::from_bits(bits);
    (value.is_finite() && (0.0..=1.0).contains(&value)).then_some(bits)
}
