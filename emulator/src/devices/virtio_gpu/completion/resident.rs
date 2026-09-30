use super::{push_used, write_response};
use crate::devices::virtio_gpu::VirtioGpu;
use crate::devices::virtio_gpu::protocol::{RESP_ERR_UNSPEC, RESP_OK_NODATA};
use crate::devices::virtio_gpu::three_d::BrowserCompletion;
use crate::memory::PhysicalMemory;

impl VirtioGpu {
    pub fn complete_3d_resident(&mut self, mem: &mut PhysicalMemory, sequence: u32) -> bool {
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
        let retired_output = pending.effect.as_ref().and_then(|effect| effect.color_target())
            .is_some_and(|(id, _)| self.pending_resource_retired(sequence, id));
        let success = pending.browser_completion == BrowserCompletion::Resident
            && pending.effect.is_some_and(|effect| {
                if retired_output {
                    self.with_pending_resources(sequence, |gpu| gpu.apply_3d_effect(effect))
                } else {
                    self.promote_resident(sequence, effect)
                }
            });
        if (!success || retired_output) && pending.browser_completion == BrowserCompletion::Resident {
            self.queue_completed_resident_release(sequence, timeline);
        }
        self.release_pending_resources(sequence);
        let response = completion.header.encode(if success { RESP_OK_NODATA } else { RESP_ERR_UNSPEC });
        let written = write_response(mem, &completion.output, &response).unwrap_or(0);
        push_used(mem, completion.used, completion.queue_size, completion.head, written as u32);
        self.interrupt_status |= 1;
        true
    }
}
