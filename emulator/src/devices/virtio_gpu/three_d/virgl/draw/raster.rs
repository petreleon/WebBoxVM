mod depth;
mod depth_textured;
mod depth_texture_color;
mod depth_vertex_color;
mod geometry;
mod solid;
mod textured;
mod texture_color;
mod vertex_color;

use super::{DrawMaterial, TextureSnapshot};
use super::super::DepthState;
use crate::devices::virtio_gpu::protocol::Rect;
use crate::devices::virtio_gpu::resource::GpuResource;

pub(super) fn valid(vertices: &[u8], material: &DrawMaterial) -> bool {
    match material {
        DrawMaterial::Solid(_) => solid::valid(vertices),
        DrawMaterial::VertexColor => vertex_color::valid(vertices),
        DrawMaterial::Textured(_) | DrawMaterial::TexturedPair(_) | DrawMaterial::ResidentTextured(_) => textured::valid(vertices),
        DrawMaterial::TextureColor(_) | DrawMaterial::ResidentTextureColor(_) => texture_color::valid(vertices),
    }
}

pub(super) fn draw_solid(
    resource: &mut GpuResource,
    rect: Rect,
    vertices: &[u8],
    color: [f32; 4],
    viewport: [f32; 6],
    scissor: Option<Rect>,
) -> bool {
    solid::draw(resource, rect, vertices, color, viewport, scissor)
}

pub(super) fn draw_depth_solid(
    resource: &mut GpuResource,
    rect: Rect,
    vertices: &[u8],
    color: [f32; 4],
    viewport: [f32; 6],
    scissor: Option<Rect>,
    state: DepthState,
    depth_values: &mut [f32],
) -> bool {
    depth::draw(resource, rect, vertices, color, viewport, scissor, state, depth_values)
}

pub(super) fn draw_depth_vertex_color(
    resource: &mut GpuResource, rect: Rect, vertices: &[u8], viewport: [f32; 6],
    scissor: Option<Rect>, state: DepthState, depth_values: &mut [f32],
) -> bool {
    depth_vertex_color::draw(resource, rect, vertices, viewport, scissor, state, depth_values)
}

pub(super) fn draw_depth_textured(
    resource: &mut GpuResource, rect: Rect, vertices: &[u8], texture: &TextureSnapshot,
    viewport: [f32; 6], scissor: Option<Rect>, state: DepthState, depth_values: &mut [f32],
) -> bool {
    depth_textured::draw(resource, rect, vertices, texture, viewport, scissor, state, depth_values)
}

pub(super) fn draw_depth_texture_color(
    resource: &mut GpuResource, rect: Rect, vertices: &[u8], texture: &TextureSnapshot,
    viewport: [f32; 6], scissor: Option<Rect>, state: DepthState, depth_values: &mut [f32],
) -> bool {
    depth_texture_color::draw(resource, rect, vertices, texture, viewport, scissor, state, depth_values)
}

pub(super) fn draw_textured(
    resource: &mut GpuResource,
    rect: Rect,
    vertices: &[u8],
    textures: &[TextureSnapshot],
    viewport: [f32; 6],
    scissor: Option<Rect>,
) -> bool {
    textured::draw(resource, rect, vertices, textures, viewport, scissor)
}

pub(super) fn draw_vertex_color(
    resource: &mut GpuResource,
    rect: Rect,
    vertices: &[u8],
    viewport: [f32; 6],
    scissor: Option<Rect>,
) -> bool {
    vertex_color::draw(resource, rect, vertices, viewport, scissor)
}

pub(super) fn draw_texture_color(
    resource: &mut GpuResource,
    rect: Rect,
    vertices: &[u8],
    texture: &TextureSnapshot,
    viewport: [f32; 6],
    scissor: Option<Rect>,
) -> bool {
    texture_color::draw(resource, rect, vertices, texture, viewport, scissor)
}
