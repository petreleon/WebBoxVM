use super::super::{BlendMode, DrawMaterial, DrawWork, MAX_VIRGL_BATCH_DRAWS, raster};
use crate::devices::virtio_gpu::VirtioGpu;
use crate::devices::virtio_gpu::protocol::Rect;
use crate::devices::virtio_gpu::resource::GpuResource;

pub(super) fn apply(
    gpu: &mut VirtioGpu,
    resource_id: u32,
    rect: Rect,
    clear: [u8; 4],
    works: Vec<DrawWork>,
) -> bool {
    if !valid(&works) { return false; }
    let drawn = {
        let Some(resource) = gpu.resources.get_mut(&resource_id) else { return false; };
        resource.clear_bgra(rect, clear).is_some() && works.iter().all(|work| draw(resource, rect, work))
    };
    if drawn { gpu.add_damage(resource_id, rect); }
    drawn
}

pub(super) fn apply_depth(
    gpu: &mut VirtioGpu,
    resource_id: u32,
    depth_resource: u32,
    rect: Rect,
    clear: [u8; 4],
    works: Vec<DrawWork>,
) -> bool {
    if !(1..=MAX_VIRGL_BATCH_DRAWS).contains(&works.len())
        || (works.len() == 1 && works[0].blend != BlendMode::Replace)
    {
        return false;
    }
    let Some(mut values) = gpu.depth_values(resource_id, depth_resource) else { return false; };
    let drawn = {
        let Some(resource) = gpu.resources.get_mut(&resource_id) else { return false; };
        resource.clear_bgra(rect, clear).is_some()
            && works.iter().all(|work| draw_depth(resource, rect, depth_resource, work, &mut values))
    };
    if !drawn || !gpu.store_depth(Some((depth_resource, values))) { return false; }
    gpu.add_damage(resource_id, rect);
    true
}

fn valid(works: &[DrawWork]) -> bool {
    (1..=MAX_VIRGL_BATCH_DRAWS).contains(&works.len())
        && works.iter().all(|work| work.blend == BlendMode::SourceOver)
}

fn draw(resource: &mut GpuResource, rect: Rect, work: &DrawWork) -> bool {
    if work.depth_resource.is_some() || work.depth_state.is_some() { return false; }
    match &work.material {
        DrawMaterial::Solid(color) => raster::draw_solid(resource, rect, &work.vertices, *color, work.viewport, work.scissor),
        DrawMaterial::VertexColor => raster::draw_vertex_color(resource, rect, &work.vertices, work.viewport, work.scissor),
        DrawMaterial::Textured(texture) => raster::draw_textured(resource, rect, &work.vertices, std::slice::from_ref(texture), work.viewport, work.scissor),
        DrawMaterial::TexturedPair(textures) => raster::draw_textured(resource, rect, &work.vertices, textures, work.viewport, work.scissor),
        DrawMaterial::TextureColor(texture) => raster::draw_texture_color(resource, rect, &work.vertices, texture, work.viewport, work.scissor),
    }
}

fn draw_depth(
    resource: &mut GpuResource,
    rect: Rect,
    depth_resource: u32,
    work: &DrawWork,
    values: &mut [f32],
) -> bool {
    let Some(state) = work.depth_state else { return false; };
    if work.depth_resource != Some(depth_resource) { return false; }
    match &work.material {
        DrawMaterial::Solid(color) => raster::draw_depth_solid(resource, rect, &work.vertices, *color, work.viewport, work.scissor, state, values),
        DrawMaterial::VertexColor => raster::draw_depth_vertex_color(resource, rect, &work.vertices, work.viewport, work.scissor, state, values),
        DrawMaterial::Textured(texture) => raster::draw_depth_textured(resource, rect, &work.vertices, texture, work.viewport, work.scissor, state, values),
        DrawMaterial::TextureColor(texture) => raster::draw_depth_texture_color(resource, rect, &work.vertices, texture, work.viewport, work.scissor, state, values),
        DrawMaterial::TexturedPair(_) => false,
    }
}
