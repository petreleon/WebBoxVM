mod blob;
mod batch;
mod clear;
mod context;
mod copy;
mod copy_buffer;
mod draw;
mod effect;
mod inline;
mod resource;
mod shader;
mod stream;
mod uniform;

use super::{BrowserCompletion, DeferredSubmit, Pending3d, Pending3dEffect};
use crate::devices::virtio_gpu::protocol::*;
use crate::devices::virtio_gpu::{MAX_PENDING_3D_BYTES, MAX_PENDING_3D_SUBMITS, VirtioGpu};

use context::DrawState;
pub(in crate::devices::virtio_gpu::three_d::virgl) use context::{
    BlendMode, FragmentConstants, SampledResource, SamplerAddressMode, SamplerConfig,
    SamplerFilter, SamplerState, UniformBinding, VertexConstants,
};
pub(in crate::devices::virtio_gpu) use context::{
    DepthCompare, DepthState, IndexBuffer, VertexBuffer, VertexElement, VirglContext,
};
pub(super) use copy::CopyRegion;
pub(super) use draw::{DrawMaterial, DrawWork};
pub(in crate::devices::virtio_gpu) use shader::ShaderKind;
#[cfg(test)]
#[allow(unused_imports)]
pub(in crate::devices::virtio_gpu) use shader::ShaderProgram;

pub(super) const VIRGL_OBJECT_DSA: u8 = 0;
pub(super) const VIRGL_OBJECT_BLEND: u8 = 1;
pub(super) const VIRGL_OBJECT_RASTERIZER: u8 = 2;
pub(super) const VIRGL_OBJECT_SHADER: u8 = 4;
pub(super) const VIRGL_OBJECT_VERTEX_ELEMENTS: u8 = 5;
pub(super) const VIRGL_OBJECT_SAMPLER_VIEW: u8 = 6;
pub(super) const VIRGL_OBJECT_SAMPLER_STATE: u8 = 7;
pub(super) const VIRGL_OBJECT_SURFACE: u8 = 8;
pub(super) const VIRGL_CMD_CLEAR_SURFACE: u8 = 62;
pub(super) const MAX_VIRGL_SUBMIT_BYTES: usize = 64 * 1024;
pub(super) const MAX_VIRGL_FRAGMENT_SAMPLERS: usize = 2;

impl VirtioGpu {
    pub(super) fn allocate_virgl_context_generation(&mut self) -> u32 {
        let generation = self.next_virgl_context_generation.max(1);
        self.next_virgl_context_generation = generation.wrapping_add(1).max(1);
        generation
    }

    pub(in crate::devices::virtio_gpu::three_d::virgl) fn queue_virgl_draw(
        &mut self,
        header: CtrlHeader,
        generation: u32,
        resource_id: u32,
        rect: Rect,
        clear: [f32; 4],
        work: DrawWork,
    ) -> Result<DeferredSubmit, u32> {
        if work.blend == BlendMode::SourceOver && work.depth_resource.is_none() && work.depth_state.is_none()
            && self.resident_target_eligible(resource_id, rect)
        {
            return self.queue_virgl_resident_singleton(header, generation, resource_id, rect, clear, work);
        }
        let sequence = self.virgl_sequence()?;
        let packet = draw::packet(sequence, rect.width, rect.height, clear, &work)
            .ok_or(RESP_ERR_INVALID_PARAMETER)?;
        self.queue_virgl_packet(
            header,
            sequence,
            packet,
            Pending3dEffect::VirglDraw {
                context_id: header.ctx_id,
                generation,
                resource_id,
                depth_resource: work.depth_resource,
                depth_state: work.depth_state,
                rect,
                clear_bgra: bgra(clear),
                material: work.material,
                vertices: work.vertices,
                viewport: work.viewport,
                scissor: work.scissor,
            },
        )
    }

    fn virgl_sequence(&mut self) -> Result<u32, u32> {
        if self.pending_3d.len() >= MAX_PENDING_3D_SUBMITS {
            return Err(RESP_ERR_OUT_OF_MEMORY);
        }
        self.allocate_3d_sequence().ok_or(RESP_ERR_OUT_OF_MEMORY)
    }

    fn queue_virgl_packet(
        &mut self,
        header: CtrlHeader,
        sequence: u32,
        packet: Vec<u8>,
        effect: Pending3dEffect,
    ) -> Result<DeferredSubmit, u32> {
        if effect.color_target().is_some_and(|(id, rect)| !self.resident_overwrite_allowed(id, rect)) {
            return Err(RESP_ERR_INVALID_PARAMETER);
        }
        if self
            .pending_3d_bytes
            .checked_add(packet.len())
            .is_none_or(|total| total > MAX_PENDING_3D_BYTES)
        {
            return Err(RESP_ERR_OUT_OF_MEMORY);
        }
        let timeline = self.fence_timeline(header);
        let browser_completion = if self.resident_candidate(&packet, &effect) {
            BrowserCompletion::Resident
        } else if matches!(packet.get(..4), Some(b"VGB1" | b"VGM1")) {
            BrowserCompletion::Readback
        } else {
            BrowserCompletion::Standard
        };
        self.pending_3d_bytes += packet.len();
        self.pending_3d.push(Pending3d {
            sequence,
            timeline,
            bytes: packet.len(),
            packet: Some(packet),
            completion: None,
            effect: Some(effect),
            browser_completion,
        });
        Ok(DeferredSubmit { sequence, header })
    }

    pub(in crate::devices::virtio_gpu) fn remove_virgl_resource(&mut self, resource_id: u32) {
        self.forget_resident(resource_id);
        self.virgl_resources.remove(&resource_id);
        for context in self.virgl_contexts.values_mut() {
            context.remove_resource(resource_id);
        }
    }

    pub(in crate::devices::virtio_gpu) fn is_virgl_resource(&self, resource_id: u32) -> bool {
        self.virgl_resources.contains(&resource_id)
    }
}

pub(super) fn bgra([red, green, blue, alpha]: [f32; 4]) -> [u8; 4] {
    [blue, green, red, alpha].map(|value| (value * 255.0).round() as u8)
}
