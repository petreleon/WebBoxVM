use super::super::ShaderProgram;

pub(super) fn parse(lines: &[&str]) -> Option<ShaderProgram> {
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
            "DCL IN[0]",
            "DCL CONST[0][0]",
            "DCL OUT[0], POSITION",
            "ADD OUT[0], IN[0], CONST[0][0]",
            "END",
        ] => Some(ShaderProgram::VertexUniformOffset),
        [
            "VERT",
            "DCL IN[0..1]",
            "DCL OUT[0], POSITION",
            "DCL OUT[1], GENERIC[0]",
            "MOV OUT[0], IN[0]",
            "MOV OUT[1], IN[1]",
            "END",
        ] => Some(ShaderProgram::VertexGeneric),
        [
            "VERT",
            "DCL IN[0..2]",
            "DCL OUT[0], POSITION",
            "DCL OUT[1], GENERIC[0]",
            "DCL OUT[2], GENERIC[1]",
            "MOV OUT[0], IN[0]",
            "MOV OUT[1], IN[1]",
            "MOV OUT[2], IN[2]",
            "END",
        ] => Some(ShaderProgram::VertexTextureColor),
        _ => None,
    }
}
