mod matrix;

use super::super::SampledResource;
use super::super::uniform;
use super::{DrawMaterial, DrawState, TextureSnapshot, solid, texture};
use super::texture::SampledTexture;
use crate::devices::virtio_gpu::VirtioGpu;
use crate::devices::virtio_gpu::protocol::RESP_ERR_INVALID_PARAMETER;
use crate::devices::virtio_gpu::three_d::virgl::VirglContext;
use crate::devices::virtio_gpu::three_d::virgl::shader::ShaderProgram;
const SOLID_VERTEX_BYTES: usize = 16;
const TEXTURED_VERTEX_BYTES: usize = 24;
const VERTEX_COLOR_BYTES: usize = 32;
const TEXTURE_COLOR_VERTEX_BYTES: usize = 40;

#[derive(Clone, Copy)]
pub(super) struct VertexTransform {
    pub(super) offset: Option<([f32; 2], usize)>,
    pub(super) matrix: Option<([f32; 16], usize)>,
    pub(super) color: Option<ColorTransform>,
}

#[derive(Clone, Copy)]
pub(super) enum ColorTransform {
    Multiply([f32; 4]),
    TextureColor([f32; 4]),
}

impl VertexTransform {
    const fn offset(offset: [f32; 2], stride: usize) -> Self {
        Self { offset: Some((offset, stride)), matrix: None, color: None }
    }

    pub(super) const fn matrix(matrix: [f32; 16], stride: usize) -> Self {
        Self { offset: None, matrix: Some((matrix, stride)), color: None }
    }

    pub(super) const fn matrix_color(matrix: [f32; 16], stride: usize, color: ColorTransform) -> Self {
        Self { offset: None, matrix: Some((matrix, stride)), color: Some(color) }
    }

    const fn multiply_color(color: [f32; 4]) -> Self {
        Self { offset: None, matrix: None, color: Some(ColorTransform::Multiply(color)) }
    }

    const fn texture_color(color: [f32; 4]) -> Self {
        Self { offset: None, matrix: None, color: Some(ColorTransform::TextureColor(color)) }
    }

    const fn offset_texture_color(offset: [f32; 2], color: [f32; 4]) -> Self {
        Self { offset: Some((offset, TEXTURED_VERTEX_BYTES)), matrix: None, color: Some(ColorTransform::TextureColor(color)) }
    }
}

pub(super) fn material(
    gpu: &VirtioGpu,
    context: &VirglContext,
    target: u32,
    state: DrawState,
) -> Result<(usize, DrawMaterial, Option<VertexTransform>), u32> {
    if matches!(state.vertex_program, ShaderProgram::VertexMatrix | ShaderProgram::VertexMatrixGeneric | ShaderProgram::VertexMatrixTextureColor) {
        return matrix::material(gpu, context, target, state);
    }
    match (state.vertex_program, state.fragment_program) {
        (ShaderProgram::VertexPassthrough, ShaderProgram::FragmentSolid(bits)) => Ok((
            SOLID_VERTEX_BYTES,
            DrawMaterial::Solid(solid::color(bits)?),
            None,
        )),
        (ShaderProgram::VertexPassthrough, ShaderProgram::FragmentConstant) => {
            let bits = uniform::resolve(gpu, context, state.fragment_constants)?;
            Ok((
                SOLID_VERTEX_BYTES,
                DrawMaterial::Solid(solid::color(bits)?),
                None,
            ))
        }
        (ShaderProgram::VertexUniformOffset, ShaderProgram::FragmentConstant) => {
            let color = solid::color(uniform::resolve(gpu, context, state.fragment_constants)?)?;
            let offset = uniform::vertex_offset(gpu, context, state.vertex_uniform)?;
            Ok((SOLID_VERTEX_BYTES, DrawMaterial::Solid(color), Some(VertexTransform::offset(offset, SOLID_VERTEX_BYTES))))
        }
        (ShaderProgram::VertexGeneric, ShaderProgram::FragmentVertexColor) => {
            Ok((VERTEX_COLOR_BYTES, DrawMaterial::VertexColor, None))
        }
        (ShaderProgram::VertexGeneric, ShaderProgram::FragmentVertexColorConstant) => {
            let color = solid::color(uniform::resolve(gpu, context, state.fragment_constants)?)?;
            Ok((VERTEX_COLOR_BYTES, DrawMaterial::VertexColor, Some(VertexTransform::multiply_color(color))))
        }
        (ShaderProgram::VertexGeneric, ShaderProgram::FragmentTextured) => Ok((
            TEXTURED_VERTEX_BYTES,
            textured(snapshot(gpu, context, target, state.sampled_resources[0])?),
            None,
        )),
        (ShaderProgram::VertexGeneric, ShaderProgram::FragmentTexturedConstant) => {
            let color = solid::color(uniform::resolve(gpu, context, state.fragment_constants)?)?;
            Ok((
                TEXTURED_VERTEX_BYTES,
                texture_color(snapshot(gpu, context, target, state.sampled_resources[0])?),
                Some(VertexTransform::texture_color(color)),
            ))
        }
        (ShaderProgram::VertexGeneric, ShaderProgram::FragmentTexturedMultiply) => Ok((
            TEXTURED_VERTEX_BYTES,
            DrawMaterial::TexturedPair(pair(gpu, context, target, state.sampled_resources)?),
            None,
        )),
        (ShaderProgram::VertexGenericUniformOffset, ShaderProgram::FragmentTextured) => {
            let offset = uniform::vertex_offset(gpu, context, state.vertex_uniform)?;
            Ok((
                TEXTURED_VERTEX_BYTES,
                textured(snapshot(gpu, context, target, state.sampled_resources[0])?),
                Some(VertexTransform::offset(offset, TEXTURED_VERTEX_BYTES)),
            ))
        }
        (ShaderProgram::VertexGenericUniformOffset, ShaderProgram::FragmentTexturedConstant) => {
            let offset = uniform::vertex_offset(gpu, context, state.vertex_uniform)?;
            let color = solid::color(uniform::resolve(gpu, context, state.fragment_constants)?)?;
            Ok((
                TEXTURED_VERTEX_BYTES,
                texture_color(snapshot(gpu, context, target, state.sampled_resources[0])?),
                Some(VertexTransform::offset_texture_color(offset, color)),
            ))
        }
        (ShaderProgram::VertexGenericUniformOffset, ShaderProgram::FragmentTexturedMultiply) => {
            let offset = uniform::vertex_offset(gpu, context, state.vertex_uniform)?;
            Ok((
                TEXTURED_VERTEX_BYTES,
                DrawMaterial::TexturedPair(pair(gpu, context, target, state.sampled_resources)?),
                Some(VertexTransform::offset(offset, TEXTURED_VERTEX_BYTES)),
            ))
        }
        (ShaderProgram::VertexTextureColor, ShaderProgram::FragmentTexturedVertexColor) => Ok((
            TEXTURE_COLOR_VERTEX_BYTES,
            texture_color(snapshot(gpu, context, target, state.sampled_resources[0])?),
            None,
        )),
        _ => Err(RESP_ERR_INVALID_PARAMETER),
    }
}

fn snapshot(
    gpu: &VirtioGpu,
    context: &VirglContext,
    target: u32,
    resource: Option<SampledResource>,
) -> Result<SampledTexture, u32> {
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
    match snapshots {
        [SampledTexture::Snapshot(left), SampledTexture::Snapshot(right)] => Ok([left, right]),
        _ => Err(RESP_ERR_INVALID_PARAMETER),
    }
}

fn textured(texture: SampledTexture) -> DrawMaterial {
    match texture {
        SampledTexture::Snapshot(texture) => DrawMaterial::Textured(texture),
        SampledTexture::Resident(texture) => DrawMaterial::ResidentTextured(texture),
    }
}

fn texture_color(texture: SampledTexture) -> DrawMaterial {
    match texture {
        SampledTexture::Snapshot(texture) => DrawMaterial::TextureColor(texture),
        SampledTexture::Resident(texture) => DrawMaterial::ResidentTextureColor(texture),
    }
}
