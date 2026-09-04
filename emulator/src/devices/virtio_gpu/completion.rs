use super::VirtioGpu;
use super::protocol::{CtrlHeader, RESP_ERR_UNSPEC, RESP_OK_NODATA};
use crate::memory::PhysicalMemory;

#[derive(Clone, Copy, Debug)]
pub(super) struct WritableRegion {
    pub addr: u64,
    pub len: u32,
}

#[derive(Clone, Debug)]
pub(super) struct PendingCompletion {
    pub header: CtrlHeader,
    pub output: Vec<WritableRegion>,
    pub used: u64,
    pub queue_size: u16,
    pub head: u16,
}

impl VirtioGpu {
    pub(super) fn attach_3d_completion(
        &mut self,
        sequence: u32,
        completion: PendingCompletion,
    ) -> bool {
        let Some(pending) = self
            .pending_3d
            .iter_mut()
            .find(|pending| pending.sequence == sequence)
        else {
            return false;
        };
        if pending.completion.is_some() {
            return false;
        }
        pending.completion = Some(completion);
        true
    }

    pub(super) fn cancel_3d(&mut self, sequence: u32) {
        if let Some(index) = self
            .pending_3d
            .iter()
            .position(|pending| pending.sequence == sequence)
        {
            let pending = self.pending_3d.remove(index);
            self.pending_3d_bytes = self.pending_3d_bytes.saturating_sub(pending.bytes);
        }
    }

    pub fn complete_3d(&mut self, mem: &mut PhysicalMemory, sequence: u32, success: bool) -> bool {
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
        let success = success
            && pending
                .effect
                .is_none_or(|effect| self.apply_3d_effect(effect));
        let response_type = if success {
            RESP_OK_NODATA
        } else {
            RESP_ERR_UNSPEC
        };
        let response = completion.header.encode(response_type);
        let written = write_response(mem, &completion.output, &response).unwrap_or(0);
        push_used(
            mem,
            completion.used,
            completion.queue_size,
            completion.head,
            written as u32,
        );
        self.interrupt_status |= 1;
        true
    }
}

pub(super) fn response_target_valid(
    mem: &PhysicalMemory,
    output: &[WritableRegion],
    required: usize,
) -> bool {
    output_capacity(output) >= required
        && output
            .iter()
            .all(|region| mem.contains_range(region.addr, region.len as usize))
}

pub(super) fn output_capacity(output: &[WritableRegion]) -> usize {
    output.iter().fold(0usize, |sum, region| {
        sum.saturating_add(region.len as usize)
    })
}

pub(super) fn write_response(
    mem: &mut PhysicalMemory,
    output: &[WritableRegion],
    data: &[u8],
) -> Option<usize> {
    if output_capacity(output) < data.len() {
        return None;
    }
    let mut done = 0;
    for region in output {
        let count = (region.len as usize).min(data.len() - done);
        if count != 0 {
            mem.write_bytes(region.addr, &data[done..done + count])?;
            done += count;
        }
        if done == data.len() {
            return Some(done);
        }
    }
    None
}

pub(super) fn push_used(mem: &mut PhysicalMemory, used: u64, count: u16, id: u16, len: u32) {
    let Some(index_addr) = used.checked_add(2) else {
        return;
    };
    let Some(used_idx) = mem.read(index_addr, 2).map(|value| value as u16) else {
        return;
    };
    let Some(elem) = used.checked_add(4 + u64::from(used_idx % count) * 8) else {
        return;
    };
    let Some(len_addr) = elem.checked_add(4) else {
        return;
    };
    let _ = mem.write(elem, 4, u64::from(id));
    let _ = mem.write(len_addr, 4, u64::from(len));
    let _ = mem.write(index_addr, 2, u64::from(used_idx.wrapping_add(1)));
}
