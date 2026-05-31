use super::super::*;

impl VirtioBlk {
    pub(super) fn read_desc(&self, mem: &PhysicalMemory, index: u16) -> Option<Descriptor> {
        if index >= self.queue_num {
            return None;
        }
        let base = self.queue_desc + index as u64 * 16;
        Some(Descriptor {
            addr: mem.read(base, 8)?,
            len: mem.read(base + 8, 4)? as u32,
            flags: mem.read(base + 12, 2)? as u16,
            next: mem.read(base + 14, 2)? as u16,
        })
    }

    pub(super) fn next_desc(&self, mem: &PhysicalMemory, desc: Descriptor) -> Option<Descriptor> {
        if desc.flags & VIRTQ_DESC_F_NEXT == 0 {
            return None;
        }
        self.read_desc(mem, desc.next)
    }

    pub(super) fn push_used(&mut self, mem: &mut PhysicalMemory, id: u32, len: u32) {
        let used_idx = mem.read(self.queue_device + 2, 2).unwrap_or(0) as u16;
        let slot = used_idx % self.queue_num;
        let elem = self.queue_device + 4 + slot as u64 * 8;
        let _ = mem.write(elem, 4, id as u64);
        let _ = mem.write(elem + 4, 4, len as u64);
        let _ = mem.write(self.queue_device + 2, 2, used_idx.wrapping_add(1) as u64);
    }
}

pub(super) fn write_bytes(mem: &mut PhysicalMemory, addr: u64, len: u32, src: &[u8]) -> (u8, u32) {
    let count = (len as usize).min(src.len());
    if count == 0 {
        return (VIRTIO_BLK_S_OK, 0);
    }
    if mem.write_bytes(addr, &src[..count]).is_none() {
        return (VIRTIO_BLK_S_IOERR, 0);
    }
    (VIRTIO_BLK_S_OK, count as u32)
}
