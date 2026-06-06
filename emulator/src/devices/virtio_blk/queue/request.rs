use super::super::*;
use super::trace::trace_request;
use crate::memory::PhysicalMemory;

struct RequestChain {
    data: Vec<Descriptor>,
    status: Descriptor,
}

impl VirtioBlk {
    pub(super) fn handle_request(&mut self, mem: &mut PhysicalMemory, head: u16) -> u32 {
        let Some(req_desc) = self.read_desc(mem, head) else {
            return 0;
        };
        let Some(chain) = self.request_chain(mem, req_desc) else {
            return 0;
        };
        let Some(req_type) = mem.read(req_desc.addr, 4).map(|v| v as u32) else {
            return 0;
        };
        let Some(sector) = mem.read(req_desc.addr + 8, 8) else {
            return 0;
        };

        let (status, written) = match req_type {
            VIRTIO_BLK_T_IN => self.read_sector_data(mem, &chain.data, sector),
            VIRTIO_BLK_T_GET_ID => write_to_descs(mem, &chain.data, self.storage.id()),
            VIRTIO_BLK_T_FLUSH => (VIRTIO_BLK_S_OK, 0),
            VIRTIO_BLK_T_OUT => self.write_sector_data(mem, &chain.data, sector),
            _ => (VIRTIO_BLK_S_UNSUPP, 0),
        };
        trace_request(self.storage.id(), req_type, sector, written, status);

        if chain.status.flags & VIRTQ_DESC_F_WRITE != 0 {
            let _ = mem.write(chain.status.addr, 1, status as u64);
        }
        written + 1
    }

    fn request_chain(&self, mem: &PhysicalMemory, req_desc: Descriptor) -> Option<RequestChain> {
        let mut data = Vec::new();
        let mut desc = self.next_desc(mem, req_desc)?;
        for _ in 0..self.queue_num {
            if desc.flags & VIRTQ_DESC_F_NEXT == 0 {
                return Some(RequestChain { data, status: desc });
            }
            data.push(desc);
            desc = self.next_desc(mem, desc)?;
        }
        None
    }

    fn read_sector_data(
        &self,
        mem: &mut PhysicalMemory,
        descs: &[Descriptor],
        sector: u64,
    ) -> (u8, u32) {
        let Some(start) = sector.checked_mul(SECTOR_SIZE as u64) else {
            return (VIRTIO_BLK_S_IOERR, 0);
        };
        let mut written = 0u32;
        for desc in descs {
            if desc.flags & VIRTQ_DESC_F_WRITE == 0 {
                return (VIRTIO_BLK_S_IOERR, 0);
            }
            let Some(offset) = start.checked_add(written as u64) else {
                return (VIRTIO_BLK_S_IOERR, 0);
            };
            let mut bytes = vec![0; desc.len as usize];
            let status = self.storage.read(offset, &mut bytes);
            if status != VIRTIO_BLK_S_OK || mem.write_bytes(desc.addr, &bytes).is_none() {
                return (VIRTIO_BLK_S_IOERR, 0);
            }
            written = written.wrapping_add(desc.len);
        }
        (VIRTIO_BLK_S_OK, written)
    }

    fn write_sector_data(
        &mut self,
        mem: &mut PhysicalMemory,
        descs: &[Descriptor],
        sector: u64,
    ) -> (u8, u32) {
        let Some(start) = sector.checked_mul(SECTOR_SIZE as u64) else {
            return (VIRTIO_BLK_S_IOERR, 0);
        };
        let mut consumed = 0u32;
        for desc in descs {
            if desc.flags & VIRTQ_DESC_F_WRITE != 0 {
                return (VIRTIO_BLK_S_IOERR, 0);
            }
            let Some(offset) = start.checked_add(consumed as u64) else {
                return (VIRTIO_BLK_S_IOERR, 0);
            };
            let mut bytes = vec![0; desc.len as usize];
            if mem.read_bytes(desc.addr, &mut bytes).is_none() {
                return (VIRTIO_BLK_S_IOERR, 0);
            }
            let status = self.storage.write(offset, &bytes);
            if status != VIRTIO_BLK_S_OK {
                return (status, 0);
            }
            consumed = consumed.wrapping_add(desc.len);
        }
        (VIRTIO_BLK_S_OK, 0)
    }
}

fn write_to_descs(mem: &mut PhysicalMemory, descs: &[Descriptor], src: &[u8]) -> (u8, u32) {
    let mut written = 0usize;
    for desc in descs {
        if desc.flags & VIRTQ_DESC_F_WRITE == 0 {
            return (VIRTIO_BLK_S_IOERR, 0);
        }
        let count = (desc.len as usize).min(src.len().saturating_sub(written));
        if count > 0
            && mem
                .write_bytes(desc.addr, &src[written..written + count])
                .is_none()
        {
            return (VIRTIO_BLK_S_IOERR, 0);
        }
        written += count;
        if written == src.len() {
            break;
        }
    }
    (VIRTIO_BLK_S_OK, written as u32)
}
