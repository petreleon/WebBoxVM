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
        let packet = super::draw::batch_packet(sequence, rect.width, rect.height, clear, &works)
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
}
