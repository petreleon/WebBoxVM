mod ring;

use super::*;
use ring::write_bytes;

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

    fn handle_request(&mut self, mem: &mut PhysicalMemory, head: u16) -> u32 {
        let Some(req_desc) = self.read_desc(mem, head) else {
            return 0;
        };
        let Some(data_desc) = self.next_desc(mem, req_desc) else {
            return 0;
        };
        let Some(status_desc) = self.next_desc(mem, data_desc) else {
            return 0;
        };
        let Some(req_type) = mem.read(req_desc.addr, 4).map(|v| v as u32) else {
            return 0;
        };
        let Some(sector) = mem.read(req_desc.addr + 8, 8) else {
            return 0;
        };

        let (status, written) = match req_type {
            VIRTIO_BLK_T_IN => self.read_sector_data(mem, data_desc, sector),
            VIRTIO_BLK_T_GET_ID => {
                write_bytes(mem, data_desc.addr, data_desc.len, self.storage.id())
            }
            VIRTIO_BLK_T_FLUSH => (VIRTIO_BLK_S_OK, 0),
            VIRTIO_BLK_T_OUT => self.write_sector_data(mem, data_desc, sector),
            _ => (VIRTIO_BLK_S_UNSUPP, 0),
        };

        if status_desc.flags & VIRTQ_DESC_F_WRITE != 0 {
            let _ = mem.write(status_desc.addr, 1, status as u64);
        }
        written + 1
    }

    fn read_sector_data(
        &self,
        mem: &mut PhysicalMemory,
        desc: Descriptor,
        sector: u64,
    ) -> (u8, u32) {
        if desc.flags & VIRTQ_DESC_F_WRITE == 0 {
            return (VIRTIO_BLK_S_IOERR, 0);
        }

        let Some(start) = sector.checked_mul(SECTOR_SIZE as u64) else {
            return (VIRTIO_BLK_S_IOERR, 0);
        };
        let mut bytes = vec![0; desc.len as usize];
        let status = self.storage.read(start, &mut bytes);
        if status == VIRTIO_BLK_S_OK {
            let _ = mem.write_bytes(desc.addr, &bytes);
        }
        (
            status,
            if status == VIRTIO_BLK_S_OK {
                desc.len
            } else {
                0
            },
        )
    }

    fn write_sector_data(
        &mut self,
        mem: &mut PhysicalMemory,
        desc: Descriptor,
        sector: u64,
    ) -> (u8, u32) {
        if desc.flags & VIRTQ_DESC_F_WRITE != 0 {
            return (VIRTIO_BLK_S_IOERR, 0);
        }

        let Some(start) = sector.checked_mul(SECTOR_SIZE as u64) else {
            return (VIRTIO_BLK_S_IOERR, 0);
        };
        let mut bytes = vec![0; desc.len as usize];
        if mem.read_bytes(desc.addr, &mut bytes).is_none() {
            return (VIRTIO_BLK_S_IOERR, 0);
        }
        (self.storage.write(start, &bytes), 0)
    }
}
