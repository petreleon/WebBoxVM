mod apply;
mod depth;
mod material;
mod packet;
mod primitive;
mod raster;
mod solid;
mod texture;
mod vertices;
use material::material;
pub(in crate::devices::virtio_gpu::three_d) use packet::{batch_packet, depth_batch_packet, packet};
pub(super) use primitive::Primitive;
use vertices::resolve;

use super::{DepthState, DrawState, SamplerConfig, VirglContext};
use crate::devices::virtio_gpu::VirtioGpu;
use crate::devices::virtio_gpu::protocol::{RESP_ERR_INVALID_PARAMETER, Rect};

pub(super) const TRIANGLE_VERTICES: u32 = 3;
pub(super) const MAX_VIRGL_DRAW_INPUT_VERTICES: u32 = 1023;
pub(super) const MAX_VIRGL_DRAW_VERTICES: u32 =
    (MAX_VIRGL_DRAW_INPUT_VERTICES - 2) * TRIANGLE_VERTICES;
pub(in crate::devices::virtio_gpu::three_d::virgl) const MAX_VIRGL_BATCH_DRAWS: usize = 16;

#[derive(Clone, Copy)]
pub(super) struct DrawCall {
    pub start: u32,
    pub count: u32,
    pub primitive: Primitive,
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
    VertexColor,
    Textured(TextureSnapshot),
    TexturedPair([TextureSnapshot; 2]),
    TextureColor(TextureSnapshot),
}

#[derive(Clone, Debug)]
pub(in crate::devices::virtio_gpu) struct DrawWork {
    pub(super) material: DrawMaterial,
    pub(super) vertices: Vec<u8>,
    pub(super) vertex_count: u32,
    pub(super) viewport: [f32; 6],
    pub(super) scissor: Option<Rect>,
    pub(super) depth_resource: Option<u32>,
    pub(super) depth_state: Option<DepthState>,
}

impl VirtioGpu {
    pub(super) fn prepare_virgl_draw(
        &self,
        context: &VirglContext,
        resource_id: u32,
        depth_resource: Option<u32>,
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
        let (vertex_bytes, material, offset) = material(self, context, resource_id, state)?;
        let depth_resource = depth::validate(
            self, context, resource_id, depth_resource, state.depth, &material,
        )?;
        let viewport = state.viewport;
        let scissor = state.scissor;
        if !viewport.valid_within(rect.width, rect.height)
            || scissor.is_some_and(|scissor| !scissor.valid_within(rect.width, rect.height))
        {
            return Err(RESP_ERR_INVALID_PARAMETER);
        }
        let mut vertices = resolve(self, context, resource_id, state, call, vertex_bytes)?;
        let vertex_count = call
            .primitive
            .output_count(call.count)
            .ok_or(RESP_ERR_INVALID_PARAMETER)?;
        let expected_bytes = usize::try_from(vertex_count)
            .ok()
            .and_then(|count| count.checked_mul(vertex_bytes))
            .ok_or(RESP_ERR_INVALID_PARAMETER)?;
        if vertices.len() != expected_bytes {
            return Err(RESP_ERR_INVALID_PARAMETER);
        }
        if let Some(offset) = offset {
            if !raster::valid(&vertices, &material) || !translate_vertices(&mut vertices, offset) {
                return Err(RESP_ERR_INVALID_PARAMETER);
            }
        }
        if !raster::valid(&vertices, &material) {
            return Err(RESP_ERR_INVALID_PARAMETER);
        }
        Ok(DrawWork {
            material,
            vertices,
            vertex_count,
            viewport: viewport.values(),
            depth_resource,
            depth_state: state.depth,
            scissor: scissor.map(|scissor| Rect {
                x: scissor.x,
                y: rect.height - (scissor.y + scissor.height),
                width: scissor.width,
                height: scissor.height,
            }),
        })
    }

}

fn translate_vertices(vertices: &mut [u8], [x, y]: [f32; 2]) -> bool {
    if !vertices.len().is_multiple_of(16) {
        return false;
    }
    for vertex in vertices.chunks_exact_mut(16) {
        for (offset, delta) in [(0, x), (4, y)] {
            let Some(bytes) = vertex.get_mut(offset..offset + 4) else {
                return false;
            };
            let value = f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) + delta;
            if !value.is_finite() {
                return false;
            }
            bytes.copy_from_slice(&value.to_le_bytes());
        }
    }
    true
}
