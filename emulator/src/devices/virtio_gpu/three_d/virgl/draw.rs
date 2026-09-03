mod material;
mod packet;
mod raster;
mod solid;
mod texture;
mod vertices;
use material::material;
pub(in crate::devices::virtio_gpu::three_d) use packet::packet;
use vertices::resolve;

use super::{DrawState, SamplerConfig, VirglContext};
use crate::devices::virtio_gpu::VirtioGpu;
use crate::devices::virtio_gpu::protocol::{RESP_ERR_INVALID_PARAMETER, Rect};

pub(super) const TRIANGLE_VERTICES: u32 = 3;

#[derive(Clone, Copy)]
pub(super) struct DrawCall {
    pub start: u32,
    pub indexed: bool,
}

#[derive(Clone, Debug)]
pub(in crate::devices::virtio_gpu) struct TextureSnapshot {
    pub width: u32,
    pub height: u32,
    pub bgra: Vec<u8>,
    pub sampler: SamplerConfig,
}

#[derive(Clone, Debug)]
pub(in crate::devices::virtio_gpu) enum DrawMaterial {
    Solid([f32; 4]),
    Textured(TextureSnapshot),
    TexturedPair([TextureSnapshot; 2]),
}

pub(super) struct DrawWork {
    pub(super) material: DrawMaterial,
    pub(super) vertices: Vec<u8>,
    pub(super) viewport: [f32; 6],
    pub(super) scissor: Option<Rect>,
}

impl VirtioGpu {
    pub(super) fn prepare_virgl_draw(
        &self,
        context: &VirglContext,
        resource_id: u32,
        rect: Rect,
        call: DrawCall,
    ) -> Result<DrawWork, u32> {
        if context.framebuffer_resource() != Some(resource_id) {
            return Err(RESP_ERR_INVALID_PARAMETER);
        }
        let target = self
            .resources
            .get(&resource_id)
            .ok_or(RESP_ERR_INVALID_PARAMETER)?;
        if !target.is_texture_2d() || !rect.valid_within(target.width, target.height) {
            return Err(RESP_ERR_INVALID_PARAMETER);
        }
        let state = context.draw_state().ok_or(RESP_ERR_INVALID_PARAMETER)?;
        let (vertex_bytes, material) = material(self, context, resource_id, state)?;
        let viewport = state.viewport;
        let scissor = state.scissor;
        if !viewport.valid_within(rect.width, rect.height)
            || scissor.is_some_and(|scissor| !scissor.valid_within(rect.width, rect.height))
        {
            return Err(RESP_ERR_INVALID_PARAMETER);
        }
        let vertices = resolve(self, context, resource_id, state, call, vertex_bytes)?;
        if !raster::valid(&vertices, &material) {
            return Err(RESP_ERR_INVALID_PARAMETER);
        }
        Ok(DrawWork {
            material,
            vertices,
            viewport: viewport.values(),
            scissor: scissor.map(|scissor| Rect {
                x: scissor.x,
                y: rect.height - (scissor.y + scissor.height),
                width: scissor.width,
                height: scissor.height,
            }),
        })
    }

    pub(super) fn apply_virgl_draw(
        &mut self,
        resource_id: u32,
        rect: Rect,
        clear: [u8; 4],
        material: DrawMaterial,
        vertices: &[u8],
        viewport: [f32; 6],
        scissor: Option<Rect>,
    ) -> bool {
        let Some(resource) = self.resources.get_mut(&resource_id) else {
            return false;
        };
        if resource.clear_bgra(rect, clear).is_none() {
            return false;
        }
        let drawn = match &material {
            DrawMaterial::Solid(color) => {
                raster::draw_solid(resource, rect, vertices, *color, viewport, scissor)
            }
            DrawMaterial::Textured(texture) => raster::draw_textured(
                resource,
                rect,
                vertices,
                std::slice::from_ref(texture),
                viewport,
                scissor,
            ),
            DrawMaterial::TexturedPair(textures) => {
                raster::draw_textured(resource, rect, vertices, textures, viewport, scissor)
            }
        };
        if !drawn {
            return false;
        }
        self.add_damage(resource_id, rect);
        true
    }
}
