use super::{ColorTransform, DrawMaterial, DrawState, TEXTURED_VERTEX_BYTES, TEXTURE_COLOR_VERTEX_BYTES,
    VERTEX_COLOR_BYTES, VertexTransform, SOLID_VERTEX_BYTES, pair, snapshot, texture_color, textured};
use super::super::super::uniform;
use crate::devices::virtio_gpu::VirtioGpu;
use crate::devices::virtio_gpu::protocol::RESP_ERR_INVALID_PARAMETER;
use crate::devices::virtio_gpu::three_d::virgl::VirglContext;
use crate::devices::virtio_gpu::three_d::virgl::shader::ShaderProgram;
use super::super::solid;

pub(super) fn material(
    gpu: &VirtioGpu,
    context: &VirglContext,
    target: u32,
    state: DrawState,
) -> Result<(usize, DrawMaterial, Option<VertexTransform>), u32> {
    let matrix = uniform::vertex_matrix(state.vertex_constants)?;
    match (state.vertex_program, state.fragment_program) {
        (ShaderProgram::VertexMatrix, ShaderProgram::FragmentSolid(bits)) => {
            solid(matrix, SOLID_VERTEX_BYTES, solid::color(bits)?)
        }
        (ShaderProgram::VertexMatrix, ShaderProgram::FragmentConstant) => {
            solid(matrix, SOLID_VERTEX_BYTES, color(gpu, context, state)?)
        }
        (ShaderProgram::VertexMatrixGeneric, ShaderProgram::FragmentVertexColor) => {
            Ok((VERTEX_COLOR_BYTES, DrawMaterial::VertexColor, Some(VertexTransform::matrix(matrix, VERTEX_COLOR_BYTES))))
        }
        (ShaderProgram::VertexMatrixGeneric, ShaderProgram::FragmentVertexColorConstant) => {
            let transform = VertexTransform::matrix_color(matrix, VERTEX_COLOR_BYTES, ColorTransform::Multiply(color(gpu, context, state)?));
            Ok((VERTEX_COLOR_BYTES, DrawMaterial::VertexColor, Some(transform)))
        }
        (ShaderProgram::VertexMatrixGeneric, ShaderProgram::FragmentTextured) => {
            let material = textured(snapshot(gpu, context, target, state.sampled_resources[0])?);
            Ok((TEXTURED_VERTEX_BYTES, material, Some(VertexTransform::matrix(matrix, TEXTURED_VERTEX_BYTES))))
        }
        (ShaderProgram::VertexMatrixGeneric, ShaderProgram::FragmentTexturedConstant) => {
            let material = texture_color(snapshot(gpu, context, target, state.sampled_resources[0])?);
            let transform = VertexTransform::matrix_color(matrix, TEXTURED_VERTEX_BYTES, ColorTransform::TextureColor(color(gpu, context, state)?));
            Ok((TEXTURED_VERTEX_BYTES, material, Some(transform)))
        }
        (ShaderProgram::VertexMatrixGeneric, ShaderProgram::FragmentTexturedMultiply) => {
            let material = DrawMaterial::TexturedPair(pair(gpu, context, target, state.sampled_resources)?);
            Ok((TEXTURED_VERTEX_BYTES, material, Some(VertexTransform::matrix(matrix, TEXTURED_VERTEX_BYTES))))
        }
        (ShaderProgram::VertexMatrixTextureColor, ShaderProgram::FragmentTexturedVertexColor) => {
            let material = texture_color(snapshot(gpu, context, target, state.sampled_resources[0])?);
            Ok((TEXTURE_COLOR_VERTEX_BYTES, material, Some(VertexTransform::matrix(matrix, TEXTURE_COLOR_VERTEX_BYTES))))
        }
        _ => Err(RESP_ERR_INVALID_PARAMETER),
    }
}

fn solid(matrix: [f32; 16], stride: usize, color: [f32; 4]) -> Result<(usize, DrawMaterial, Option<VertexTransform>), u32> {
    Ok((stride, DrawMaterial::Solid(color), Some(VertexTransform::matrix(matrix, stride))))
}

fn color(gpu: &VirtioGpu, context: &VirglContext, state: DrawState) -> Result<[f32; 4], u32> {
    solid::color(uniform::resolve(gpu, context, state.fragment_constants)?)
}
