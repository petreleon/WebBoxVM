use super::DrawWork;
use super::super::{DeferredSubmit, Pending3dEffect};
use crate::devices::virtio_gpu::VirtioGpu;
use crate::devices::virtio_gpu::protocol::{CtrlHeader, RESP_ERR_INVALID_PARAMETER, Rect};

impl VirtioGpu {
    pub(in crate::devices::virtio_gpu::three_d::virgl) fn queue_virgl_batch(
        &mut self,
        header: CtrlHeader,
        generation: u32,
        resource_id: u32,
        rect: Rect,
        clear: [f32; 4],
        works: Vec<DrawWork>,
    ) -> Result<DeferredSubmit, u32> {
        let sequence = self.virgl_sequence()?;
        let resident = self.resident_target_eligible(resource_id, rect);
        let packet = super::draw::batch_packet(sequence, rect.width, rect.height, clear, &works, resident)
            .or_else(|| super::draw::material_batch_packet(sequence, rect.width, rect.height, clear, &works, false))
            .ok_or(RESP_ERR_INVALID_PARAMETER)?;
        self.queue_virgl_packet(
            header,
            sequence,
            packet,
            Pending3dEffect::VirglBatch {
                context_id: header.ctx_id,
                generation,
                resource_id,
                rect,
                clear_bgra: super::bgra(clear),
                works,
            },
        )
    }

    pub(in crate::devices::virtio_gpu::three_d::virgl) fn queue_virgl_depth_batch(
        &mut self,
        header: CtrlHeader,
        generation: u32,
        resource_id: u32,
        depth_resource: u32,
        rect: Rect,
        clear: [f32; 4],
        works: Vec<DrawWork>,
    ) -> Result<DeferredSubmit, u32> {
        if !works.iter().all(|work| {
            work.depth_resource == Some(depth_resource) && work.depth_state.is_some()
        }) {
            return Err(RESP_ERR_INVALID_PARAMETER);
        }
        let sequence = self.virgl_sequence()?;
        let packet = super::draw::depth_batch_packet(sequence, rect.width, rect.height, clear, &works)
            .or_else(|| super::draw::material_batch_packet(sequence, rect.width, rect.height, clear, &works, true))
            .ok_or(RESP_ERR_INVALID_PARAMETER)?;
        self.queue_virgl_packet(
            header,
            sequence,
            packet,
            Pending3dEffect::VirglDepthBatch {
                context_id: header.ctx_id, generation, resource_id, depth_resource, rect,
                clear_bgra: super::bgra(clear), works,
            },
        )
    }
}
