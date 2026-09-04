use super::shape::{self, Operation};
use super::super::ShaderProgram;

pub(super) fn parse(lines: &[&str]) -> Option<ShaderProgram> {
    let shape = shape::parse(lines, "VERT")?;
    match shape.operations() {
        [Operation::Mov(output, input), Operation::End]
            if position(&shape) && shape::same(output, "OUT[0]") && shape::same(input, "IN[0]") =>
        {
            Some(ShaderProgram::VertexPassthrough)
        }
        [Operation::Add(output, left, right), Operation::End]
            if position(&shape) && shape.has_register("CONST[0][0]")
                && shape::same(output, "OUT[0]") && sum(left, right, "IN[0]", "CONST[0][0]") =>
        {
            Some(ShaderProgram::VertexUniformOffset)
        }
        [Operation::Mov(position, input), Operation::Mov(generic_output, varying), Operation::End]
            if generic(&shape, 1) && shape::same(position, "OUT[0]") && shape::same(input, "IN[0]")
                && shape::same(generic_output, "OUT[1]") && shape::same(varying, "IN[1]") =>
        {
            Some(ShaderProgram::VertexGeneric)
        }
        [Operation::Add(position, left, right), Operation::Mov(generic_output, varying), Operation::End]
            if generic(&shape, 1) && shape.has_register("CONST[0][0]") && shape::same(position, "OUT[0]")
                && sum(left, right, "IN[0]", "CONST[0][0]") && shape::same(generic_output, "OUT[1]")
                && shape::same(varying, "IN[1]") =>
        {
            Some(ShaderProgram::VertexGenericUniformOffset)
        }
        [Operation::Mov(position, input), Operation::Mov(generic0, varying0), Operation::Mov(generic1, varying1), Operation::End]
            if generic(&shape, 2) && shape::same(position, "OUT[0]") && shape::same(input, "IN[0]")
                && shape::same(generic0, "OUT[1]") && shape::same(varying0, "IN[1]")
                && shape::same(generic1, "OUT[2]") && shape::same(varying1, "IN[2]") =>
        {
            Some(ShaderProgram::VertexTextureColor)
        }
        _ => None,
    }
}

fn position(shape: &shape::Shape<'_>) -> bool {
    shape.has_optional_semantic("IN[0]", "POSITION") && shape.has_semantic("OUT[0]", "POSITION")
}

fn generic(shape: &shape::Shape<'_>, count: u32) -> bool {
    position(shape) && (1..=count).all(|index| {
        shape.has_optional_semantic(&format!("IN[{index}]"), &format!("GENERIC[{}]", index - 1))
            && shape.has_semantic(&format!("OUT[{index}]"), &format!("GENERIC[{}]", index - 1))
    })
}

fn sum(left: &str, right: &str, first: &str, second: &str) -> bool {
    (shape::same(left, first) && shape::same(right, second))
        || (shape::same(left, second) && shape::same(right, first))
}
