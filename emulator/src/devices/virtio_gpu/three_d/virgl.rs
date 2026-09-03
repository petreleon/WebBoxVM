mod context;
mod resource;
mod stream;

use super::{DeferredSubmit, Pending3d, Pending3dEffect};
use crate::devices::virtio_gpu::protocol::*;
use crate::devices::virtio_gpu::{MAX_PENDING_3D_BYTES, MAX_PENDING_3D_SUBMITS, VirtioGpu};

pub(in crate::devices::virtio_gpu) use context::VirglContext;

pub(super) const VIRGL_OBJECT_SURFACE: u8 = 7;
pub(super) const VIRGL_CMD_CLEAR_SURFACE: u8 = 62;
pub(super) const MAX_VIRGL_SUBMIT_BYTES: usize = 64 * 1024;

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
        if self.pending_3d.len() >= MAX_PENDING_3D_SUBMITS {
            return Err(RESP_ERR_OUT_OF_MEMORY);
        }
        let sequence = self.allocate_3d_sequence().ok_or(RESP_ERR_OUT_OF_MEMORY)?;
        let packet = clear_packet(sequence, rect.width, rect.height, color);
        if self
            .pending_3d_bytes
            .checked_add(packet.len())
            .is_none_or(|total| total > MAX_PENDING_3D_BYTES)
        {
            return Err(RESP_ERR_OUT_OF_MEMORY);
        }
        self.pending_3d_bytes += packet.len();
        self.pending_3d.push(Pending3d {
            sequence,
            bytes: packet.len(),
            packet: Some(packet),
            completion: None,
            effect: Some(Pending3dEffect::VirglClear {
                context_id: header.ctx_id,
                generation,
                resource_id,
                rect,
                bgra: bgra(color),
            }),
        });
        Ok(DeferredSubmit { sequence, header })
    }

    pub(in crate::devices::virtio_gpu) fn apply_3d_effect(
        &mut self,
        effect: Pending3dEffect,
    ) -> bool {
        let Pending3dEffect::VirglClear {
            context_id,
            generation,
            resource_id,
            rect,
            bgra,
        } = effect;
        if self
            .virgl_contexts
            .get(&context_id)
            .map(|ctx| ctx.generation)
            != Some(generation)
        {
            return false;
        }
        let cleared = self
            .resources
            .get_mut(&resource_id)
            .is_some_and(|resource| resource.clear_bgra(rect, bgra).is_some());
        if cleared {
            self.add_damage(resource_id, rect);
        }
        cleared
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
