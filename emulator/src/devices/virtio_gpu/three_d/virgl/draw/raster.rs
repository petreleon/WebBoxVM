mod geometry;
mod solid;
mod textured;
mod vertex_color;

use super::{DrawMaterial, TextureSnapshot};
use crate::devices::virtio_gpu::protocol::Rect;
use crate::devices::virtio_gpu::resource::GpuResource;

pub(super) fn valid(vertices: &[u8], material: &DrawMaterial) -> bool {
    match material {
        DrawMaterial::Solid(_) => solid::valid(vertices),
        DrawMaterial::VertexColor => vertex_color::valid(vertices),
        DrawMaterial::Textured(_) | DrawMaterial::TexturedPair(_) => textured::valid(vertices),
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
