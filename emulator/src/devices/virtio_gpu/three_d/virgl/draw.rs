mod raster;

use super::shader::ShaderProgram;
use super::{DrawState, VirglContext};
use crate::devices::virtio_gpu::VirtioGpu;
use crate::devices::virtio_gpu::protocol::{RESP_ERR_INVALID_PARAMETER, Rect};
use crate::devices::virtio_gpu::resource::FORMAT_R32G32B32A32_FLOAT;

const TRIANGLE_VERTICES: u32 = 3;
const VERTEX_BYTES: usize = 16;

#[derive(Clone, Copy)]
pub(super) struct DrawCall {
    pub start: u32,
}

pub(super) struct DrawWork {
    pub color: [f32; 4],
    pub vertices: Vec<u8>,
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
        let color = draw_color(state)?;
        let binding = state.vertex_buffer;
        let element = state.vertex_element;
        let source = self
            .resources
            .get(&binding.resource)
            .ok_or(RESP_ERR_INVALID_PARAMETER)?;
        let valid = binding.stride == VERTEX_BYTES as u32
            && binding.offset % VERTEX_BYTES as u32 == 0
            && element.offset == 0
            && element.divisor == 0
            && element.buffer_index == 0
            && element.format == FORMAT_R32G32B32A32_FLOAT
            && source.is_buffer()
            && source.format == FORMAT_R32G32B32A32_FLOAT
            && context.is_attached(resource_id)
            && context.is_attached(binding.resource)
            && self.is_virgl_resource(resource_id)
            && self.is_virgl_resource(binding.resource);
        if !valid {
            return Err(RESP_ERR_INVALID_PARAMETER);
        }
        let start = usize::try_from(call.start).map_err(|_| RESP_ERR_INVALID_PARAMETER)?;
        let offset = usize::try_from(binding.offset).map_err(|_| RESP_ERR_INVALID_PARAMETER)?;
        let bytes = usize::try_from(TRIANGLE_VERTICES)
            .ok()
            .and_then(|count| count.checked_mul(VERTEX_BYTES))
            .ok_or(RESP_ERR_INVALID_PARAMETER)?;
        let start = start
            .checked_mul(VERTEX_BYTES)
            .and_then(|value| value.checked_add(offset))
            .ok_or(RESP_ERR_INVALID_PARAMETER)?;
        let vertices = source
            .pixels
            .get(start..start.checked_add(bytes).ok_or(RESP_ERR_INVALID_PARAMETER)?)
            .ok_or(RESP_ERR_INVALID_PARAMETER)?
            .to_vec();
        if !raster::valid(&vertices) {
            return Err(RESP_ERR_INVALID_PARAMETER);
        }
        Ok(DrawWork { color, vertices })
    }

    pub(super) fn apply_virgl_draw(
        &mut self,
        resource_id: u32,
        rect: Rect,
        clear: [u8; 4],
        color: [f32; 4],
        vertices: &[u8],
    ) -> bool {
        let Some(resource) = self.resources.get_mut(&resource_id) else {
            return false;
        };
        if resource.clear_bgra(rect, clear).is_none()
            || !raster::draw(resource, rect, vertices, color)
        {
            return false;
        }
        self.add_damage(resource_id, rect);
        true
    }
}

pub(super) fn packet(
    sequence: u32,
    width: u32,
    height: u32,
    clear: [f32; 4],
    work: &DrawWork,
) -> Vec<u8> {
    let mut packet = b"VGD1".to_vec();
    for value in [1, sequence, width, height, TRIANGLE_VERTICES] {
        packet.extend_from_slice(&value.to_le_bytes());
    }
    for value in clear.into_iter().chain(work.color) {
        packet.extend_from_slice(&value.to_le_bytes());
    }
    packet.extend_from_slice(&work.vertices);
    packet
}

fn draw_color(state: DrawState) -> Result<[f32; 4], u32> {
    if state.vertex_program != ShaderProgram::VertexPassthrough {
        return Err(RESP_ERR_INVALID_PARAMETER);
    }
    let ShaderProgram::FragmentSolid(bits) = state.fragment_program else {
        return Err(RESP_ERR_INVALID_PARAMETER);
    };
    let color = bits.map(f32::from_bits);
    color
        .iter()
        .all(|value| value.is_finite() && (0.0..=1.0).contains(value))
        .then_some(color)
        .ok_or(RESP_ERR_INVALID_PARAMETER)
}
