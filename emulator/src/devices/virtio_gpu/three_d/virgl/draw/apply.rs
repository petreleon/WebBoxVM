mod batch;

use super::{DepthState, DrawMaterial, DrawWork, raster};
use crate::devices::virtio_gpu::VirtioGpu;
use crate::devices::virtio_gpu::protocol::Rect;

impl VirtioGpu {
    pub(in crate::devices::virtio_gpu) fn apply_virgl_batch(
        &mut self,
        resource_id: u32,
        rect: Rect,
        clear: [u8; 4],
        works: Vec<DrawWork>,
    ) -> bool {
        batch::apply(self, resource_id, rect, clear, works)
    }

    pub(in crate::devices::virtio_gpu) fn apply_virgl_depth_batch(
        &mut self,
        resource_id: u32,
        depth_resource: u32,
        rect: Rect,
        clear: [u8; 4],
        works: Vec<DrawWork>,
    ) -> bool {
        batch::apply_depth(self, resource_id, depth_resource, rect, clear, works)
    }

    pub(in crate::devices::virtio_gpu) fn apply_virgl_draw(
        &mut self,
        resource_id: u32,
        depth_resource: Option<u32>,
        depth_state: Option<DepthState>,
        rect: Rect,
        clear: [u8; 4],
        material: DrawMaterial,
        vertices: &[u8],
        viewport: [f32; 6],
        scissor: Option<Rect>,
    ) -> bool {
        let mut depth = match (depth_resource, depth_state) {
            (Some(depth_resource), Some(state)) => self
                .depth_values(resource_id, depth_resource)
                .map(|values| (depth_resource, state, values)),
            (None, None) => None,
            _ => return false,
        };
        let drawn = {
            let Some(resource) = self.resources.get_mut(&resource_id) else { return false; };
            if resource.clear_bgra(rect, clear).is_none() { return false; }
            match (&material, depth.as_mut()) {
                (DrawMaterial::Solid(color), Some((_, state, values))) => {
                    raster::draw_depth_solid(resource, rect, vertices, *color, viewport, scissor, *state, values)
                }
                (DrawMaterial::VertexColor, Some((_, state, values))) => {
                    raster::draw_depth_vertex_color(resource, rect, vertices, viewport, scissor, *state, values)
                }
                (DrawMaterial::Solid(color), None) => raster::draw_solid(resource, rect, vertices, *color, viewport, scissor),
                (DrawMaterial::VertexColor, None) => raster::draw_vertex_color(resource, rect, vertices, viewport, scissor),
                (DrawMaterial::Textured(texture), None) => raster::draw_textured(resource, rect, vertices, std::slice::from_ref(texture), viewport, scissor),
                (DrawMaterial::TexturedPair(textures), None) => raster::draw_textured(resource, rect, vertices, textures, viewport, scissor),
                (DrawMaterial::TextureColor(texture), None) => raster::draw_texture_color(resource, rect, vertices, texture, viewport, scissor),
                (_, Some(_)) => false,
            }
        };
        if !drawn || !self.store_depth(depth.map(|(id, _, values)| (id, values))) { return false; }
        self.add_damage(resource_id, rect);
        true
    }

    fn depth_values(&self, color_id: u32, depth_id: u32) -> Option<Vec<f32>> {
        let color = self.resources.get(&color_id)?;
        let depth = self.resources.get(&depth_id)?;
        (color_id != depth_id
            && depth.is_depth_texture_2d()
            && color.width == depth.width
            && color.height == depth.height
            && color.pixels.len() == depth.pixels.len())
            .then(|| vec![1.0; depth.pixels.len() / 4])
    }

    fn store_depth(&mut self, depth: Option<(u32, Vec<f32>)>) -> bool {
        let Some((resource_id, values)) = depth else { return true; };
        let Some(resource) = self.resources.get_mut(&resource_id) else { return false; };
        if !resource.is_depth_texture_2d() || values.len() * 4 != resource.pixels.len() { return false; }
        for (pixel, value) in resource.pixels.chunks_exact_mut(4).zip(values) {
            pixel.copy_from_slice(&value.to_le_bytes());
        }
        true
    }
}
