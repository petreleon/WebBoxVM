use super::{push_used, write_response};
use crate::devices::virtio_gpu::three_d::{BrowserCompletion, Pending3dEffect};
use crate::devices::virtio_gpu::VirtioGpu;
use crate::devices::virtio_gpu::protocol::{RESP_ERR_UNSPEC, RESP_OK_NODATA};
use crate::memory::PhysicalMemory;

impl VirtioGpu {
    pub fn complete_3d_readback(
        &mut self,
        mem: &mut PhysicalMemory,
        sequence: u32,
        format: u32,
        pixels: &[u8],
    ) -> bool {
        let Some(index) = self.pending_3d.iter().position(|pending| {
            pending.sequence == sequence && pending.packet.is_none() && pending.completion.is_some()
        }) else {
            return false;
        };
        let timeline = self.pending_3d[index].timeline;
        if self.pending_3d[..index]
            .iter()
            .any(|pending| pending.timeline == timeline && pending.completion.is_some())
        {
            return false;
        }
        let pending = self.pending_3d.remove(index);
        self.pending_3d_bytes = self.pending_3d_bytes.saturating_sub(pending.bytes);
        let completion = pending.completion.expect("completion checked above");
        let output = pending.effect.as_ref().and_then(|effect| effect.color_target().map(|(id, _)| id));
        let retired_output = output.is_some_and(|id| self.pending_resource_retired(sequence, id));
        let success = matches!(pending.browser_completion, BrowserCompletion::Readback | BrowserCompletion::Resident)
            && pending.effect.is_some_and(|effect| self.with_pending_resources(sequence, |gpu| {
                if matches!(&effect, Pending3dEffect::VirglResidentReadback { .. }) {
                    gpu.resolve_resident_readback(mem, effect, format, pixels)
                } else {
                    gpu.apply_3d_readback(effect, format, pixels)
                }
            }));
        if success && !retired_output {
            if let Some(resource_id) = output {
                self.forget_completed_resident(resource_id, timeline);
            }
        }
        self.release_pending_resources(sequence, timeline);
        let response = completion.header.encode(if success { RESP_OK_NODATA } else { RESP_ERR_UNSPEC });
        let written = write_response(mem, &completion.output, &response).unwrap_or(0);
        push_used(mem, completion.used, completion.queue_size, completion.head, written as u32);
        self.interrupt_status |= 1;
        true
    }
}
