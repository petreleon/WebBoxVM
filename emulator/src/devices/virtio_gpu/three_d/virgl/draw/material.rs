use super::super::SampledResource;
use super::{DrawMaterial, DrawState, TextureSnapshot, solid, texture};
use crate::devices::virtio_gpu::VirtioGpu;
use crate::devices::virtio_gpu::protocol::RESP_ERR_INVALID_PARAMETER;
use crate::devices::virtio_gpu::three_d::virgl::VirglContext;
use crate::devices::virtio_gpu::three_d::virgl::shader::ShaderProgram;

const SOLID_VERTEX_BYTES: usize = 16;
const TEXTURED_VERTEX_BYTES: usize = 24;
const VERTEX_COLOR_BYTES: usize = 32;
const TEXTURE_COLOR_VERTEX_BYTES: usize = 40;

pub(super) fn material(
    gpu: &VirtioGpu,
    context: &VirglContext,
    target: u32,
    state: DrawState,
) -> Result<(usize, DrawMaterial), u32> {
    match (state.vertex_program, state.fragment_program) {
        (ShaderProgram::VertexPassthrough, ShaderProgram::FragmentSolid(bits)) => {
            Ok((SOLID_VERTEX_BYTES, DrawMaterial::Solid(solid::color(bits)?)))
        }
        (ShaderProgram::VertexPassthrough, ShaderProgram::FragmentConstant) => {
            let bits = state.fragment_constants.ok_or(RESP_ERR_INVALID_PARAMETER)?;
            Ok((SOLID_VERTEX_BYTES, DrawMaterial::Solid(solid::color(bits)?)))
        }
        (ShaderProgram::VertexGeneric, ShaderProgram::FragmentVertexColor) => {
            Ok((VERTEX_COLOR_BYTES, DrawMaterial::VertexColor))
        }
        (ShaderProgram::VertexGeneric, ShaderProgram::FragmentTextured) => Ok((
            TEXTURED_VERTEX_BYTES,
            DrawMaterial::Textured(snapshot(gpu, context, target, state.sampled_resources[0])?),
        )),
        (ShaderProgram::VertexGeneric, ShaderProgram::FragmentTexturedMultiply) => Ok((
            TEXTURED_VERTEX_BYTES,
            DrawMaterial::TexturedPair(pair(gpu, context, target, state.sampled_resources)?),
        )),
        (ShaderProgram::VertexTextureColor, ShaderProgram::FragmentTexturedVertexColor) => Ok((
            TEXTURE_COLOR_VERTEX_BYTES,
            DrawMaterial::TextureColor(snapshot(gpu, context, target, state.sampled_resources[0])?),
        )),
        _ => Err(RESP_ERR_INVALID_PARAMETER),
    }
}

fn snapshot(
    gpu: &VirtioGpu,
    context: &VirglContext,
    target: u32,
    resource: Option<SampledResource>,
) -> Result<TextureSnapshot, u32> {
    texture::snapshot(gpu, context, target, resource)
}

fn pair(
    gpu: &VirtioGpu,
    context: &VirglContext,
    target: u32,
    [left, right]: [Option<SampledResource>; 2],
) -> Result<[TextureSnapshot; 2], u32> {
    let snapshots = [
        snapshot(gpu, context, target, left)?,
        snapshot(gpu, context, target, right)?,
    ];
    Ok(snapshots)
}
