mod blob;
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

use super::{DeferredSubmit, Pending3d, Pending3dEffect};
use crate::devices::virtio_gpu::protocol::*;
use crate::devices::virtio_gpu::{MAX_PENDING_3D_BYTES, MAX_PENDING_3D_SUBMITS, VirtioGpu};

use context::DrawState;
pub(in crate::devices::virtio_gpu::three_d::virgl) use context::{
    FragmentConstants, SampledResource, SamplerAddressMode, SamplerConfig, SamplerFilter,
    SamplerState, UniformBinding,
};
pub(in crate::devices::virtio_gpu) use context::{
    IndexBuffer, VertexBuffer, VertexElement, VirglContext,
};
pub(super) use copy::CopyRegion;
pub(super) use draw::DrawMaterial;
use draw::DrawWork;
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

    pub(super) fn queue_virgl_clear(
        &mut self,
        header: CtrlHeader,
        generation: u32,
        resource_id: u32,
        rect: Rect,
        color: [f32; 4],
    ) -> Result<DeferredSubmit, u32> {
        let sequence = self.virgl_sequence()?;
        let packet = clear_packet(sequence, rect.width, rect.height, color);
        self.queue_virgl_packet(
            header,
            sequence,
            packet,
            Pending3dEffect::VirglClear {
                context_id: header.ctx_id,
                generation,
                resource_id,
                rect,
                bgra: bgra(color),
            },
        )
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
        let sequence = self.virgl_sequence()?;
        let packet = draw::packet(sequence, rect.width, rect.height, clear, &work);
        self.queue_virgl_packet(
            header,
            sequence,
            packet,
            Pending3dEffect::VirglDraw {
                context_id: header.ctx_id,
                generation,
                resource_id,
                depth_resource: work.depth_resource,
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
        if self
            .pending_3d_bytes
            .checked_add(packet.len())
            .is_none_or(|total| total > MAX_PENDING_3D_BYTES)
        {
            return Err(RESP_ERR_OUT_OF_MEMORY);
        }
        let timeline = self.fence_timeline(header);
        self.pending_3d_bytes += packet.len();
        self.pending_3d.push(Pending3d {
            sequence,
            timeline,
            bytes: packet.len(),
            packet: Some(packet),
            completion: None,
            effect: Some(effect),
        });
        Ok(DeferredSubmit { sequence, header })
    }

    pub(in crate::devices::virtio_gpu) fn remove_virgl_resource(&mut self, resource_id: u32) {
        self.virgl_resources.remove(&resource_id);
        for context in self.virgl_contexts.values_mut() {
            context.remove_resource(resource_id);
        }
    }

    pub(in crate::devices::virtio_gpu) fn is_virgl_resource(&self, resource_id: u32) -> bool {
        self.virgl_resources.contains(&resource_id)
    }
}

pub(super) fn clear_packet(sequence: u32, width: u32, height: u32, color: [f32; 4]) -> Vec<u8> {
    let mut packet = b"VGC1".to_vec();
    for value in [1, sequence, width, height] {
        packet.extend_from_slice(&value.to_le_bytes());
    }
    for value in color {
        packet.extend_from_slice(&value.to_le_bytes());
    }
    packet
}

pub(super) fn bgra([red, green, blue, alpha]: [f32; 4]) -> [u8; 4] {
    [blue, green, red, alpha].map(|value| (value * 255.0).round() as u8)
}
