use super::{generic, position};
use super::super::super::ShaderProgram;
use super::super::shape::{self, Operation};

pub(super) fn parse(shape: &shape::Shape<'_>) -> Option<ShaderProgram> {
    let mut rows = [false; 4];
    let mut moves = Vec::new();
    for operation in shape.operations() {
        match *operation {
            Operation::Dp4(output, input, constant) => {
                let index = component(output)?;
                if rows[index] || !shape::same(input, "IN[0]")
                    || !shape::same(constant, &format!("CONST[{index}]"))
                {
                    return None;
                }
                rows[index] = true;
            }
            Operation::Mov(output, input) => moves.push((output, input)),
            Operation::End => {}
            _ => return None,
        }
    }
    if !position(shape) || !rows.into_iter().all(|found| found)
        || !(0..4).all(|index| shape.has_register(&format!("CONST[{index}]")))
    {
        return None;
    }
    match moves.as_slice() {
        [] => Some(ShaderProgram::VertexMatrix),
        [(output, input)] if generic(shape, 1) && shape::same(output, "OUT[1]")
            && shape::same(input, "IN[1]") => Some(ShaderProgram::VertexMatrixGeneric),
        [(first_output, first_input), (second_output, second_input)]
            if generic(shape, 2) && shape::same(first_output, "OUT[1]")
                && shape::same(first_input, "IN[1]") && shape::same(second_output, "OUT[2]")
                && shape::same(second_input, "IN[2]") =>
        {
            Some(ShaderProgram::VertexMatrixTextureColor)
        }
        _ => None,
    }
}

fn component(output: &str) -> Option<usize> {
    ['x', 'y', 'z', 'w']
        .into_iter()
        .position(|component| shape::component_matches(output, "OUT[0]", component))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_dp4_rows_form_a_vertex_matrix_program() {
        let lines = ["VERT", "DCL OUT[0], POSITION", "DCL CONST[0..3]", "DCL IN[0]",
            "DP4 OUT[0].x, IN[0], CONST[0]", "DP4 OUT[0].y, IN[0], CONST[1]",
            "DP4 OUT[0].z, IN[0], CONST[2]", "DP4 OUT[0].w, IN[0], CONST[3]", "END"];
        assert_eq!(super::super::parse(&lines), Some(ShaderProgram::VertexMatrix));
    }

    #[test]
    fn matrix_generic_normalizes_varying_and_dp4_order() {
        let lines = ["VERT", "DCL OUT[1], GENERIC[0]", "DCL IN[0], POSITION", "DCL CONST[0..3]",
            "DCL IN[1], GENERIC[0]", "DCL OUT[0], POSITION", "MOV OUT[1], IN[1]",
            "DP4 OUT[0].w, IN[0], CONST[3]", "DP4 OUT[0].x, IN[0], CONST[0]",
            "DP4 OUT[0].z, IN[0], CONST[2]", "DP4 OUT[0].y, IN[0], CONST[1]", "END"];
        assert_eq!(super::super::parse(&lines), Some(ShaderProgram::VertexMatrixGeneric));
    }

    #[test]
    fn dp4_matrix_rejects_a_wrong_output_component() {
        let lines = ["VERT", "DCL IN[0]", "DCL CONST[0..3]", "DCL OUT[0], POSITION",
            "DP4 OUT[0].x, IN[0], CONST[0]", "DP4 OUT[0].y, IN[0], CONST[1]",
            "DP4 OUT[0].z, IN[0], CONST[2]", "DP4 OUT[0].z, IN[0], CONST[3]", "END"];
        assert_eq!(super::super::parse(&lines), None);
    }
}
