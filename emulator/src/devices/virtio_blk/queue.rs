mod request;
mod ring;

use super::*;

impl VirtioBlk {
    pub(super) fn process_queue(&mut self, mem: &mut PhysicalMemory) -> bool {
        if !self.queue_ready || self.queue_num == 0 || self.queue_desc == 0 {
            return false;
        }

        let Some(avail_idx) = mem.read(self.queue_driver + 2, 2).map(|v| v as u16) else {
            return false;
        };

        let mut completed = false;
        while self.last_avail_idx != avail_idx {
            let ring_slot = self.last_avail_idx % self.queue_num;
            let Some(head) = mem
                .read(self.queue_driver + 4 + ring_slot as u64 * 2, 2)
                .map(|v| v as u16)
            else {
                break;
            };
            let written = self.handle_request(mem, head);
            self.push_used(mem, head as u32, written);
            self.last_avail_idx = self.last_avail_idx.wrapping_add(1);
            completed = true;
        }

        if completed {
            self.interrupt_status |= 1;
        }
        completed
    }
}
