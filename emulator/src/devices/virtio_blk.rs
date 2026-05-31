//! Minimal read-only VirtIO-MMIO block device.
//!
//! This implements the small subset Linux needs to read ISO sectors through
//! `virtio_blk`: feature negotiation, one split virtqueue, and read requests.

use crate::memory::PhysicalMemory;

const VIRTIO_MMIO_MAGIC: u64 = 0x7472_6976;
const VIRTIO_MMIO_VERSION_2: u64 = 2;
const VIRTIO_DEVICE_ID_BLOCK: u64 = 2;
const VIRTIO_VENDOR_WEBBOXVM: u64 = 0x5742_564d;

const VIRTIO_F_VERSION_1: u64 = 1 << 32;

const VIRTQ_DESC_F_NEXT: u16 = 1;
const VIRTQ_DESC_F_WRITE: u16 = 2;

const VIRTIO_BLK_T_IN: u32 = 0;
const VIRTIO_BLK_T_OUT: u32 = 1;
const VIRTIO_BLK_T_FLUSH: u32 = 4;
const VIRTIO_BLK_T_GET_ID: u32 = 8;

const VIRTIO_BLK_S_OK: u8 = 0;
const VIRTIO_BLK_S_IOERR: u8 = 1;
const VIRTIO_BLK_S_UNSUPP: u8 = 2;

const SECTOR_SIZE: usize = 512;
const QUEUE_NUM_MAX: u16 = 64;

#[derive(Clone, Copy, Debug)]
struct Descriptor {
    addr: u64,
    len: u32,
    flags: u16,
    next: u16,
}

#[derive(Debug, Clone)]
pub struct VirtioBlk {
    image: Vec<u8>,
    device_features_sel: u32,
    driver_features_sel: u32,
    queue_sel: u32,
    queue_num: u16,
    queue_ready: bool,
    queue_desc: u64,
    queue_driver: u64,
    queue_device: u64,
    last_avail_idx: u16,
    interrupt_status: u32,
    status: u32,
}

impl VirtioBlk {
    pub fn new() -> Self {
        Self {
            image: Vec::new(),
            device_features_sel: 0,
            driver_features_sel: 0,
            queue_sel: 0,
            queue_num: 0,
            queue_ready: false,
            queue_desc: 0,
            queue_driver: 0,
            queue_device: 0,
            last_avail_idx: 0,
            interrupt_status: 0,
            status: 0,
        }
    }

    pub fn set_image(&mut self, image: &[u8]) {
        self.image.clear();
        self.image.extend_from_slice(image);
    }

    pub fn read(&self, offset: u64, size: u8) -> Option<u64> {
        let val = match offset {
            0x000 => VIRTIO_MMIO_MAGIC,
            0x004 => VIRTIO_MMIO_VERSION_2,
            0x008 => VIRTIO_DEVICE_ID_BLOCK,
            0x00c => VIRTIO_VENDOR_WEBBOXVM,
            0x010 => self.selected_device_features(),
            0x034 => QUEUE_NUM_MAX as u64,
            0x044 => self.queue_ready as u64,
            0x060 => self.interrupt_status as u64,
            0x070 => self.status as u64,
            0x080 => self.queue_desc as u32 as u64,
            0x084 => self.queue_desc >> 32,
            0x090 => self.queue_driver as u32 as u64,
            0x094 => self.queue_driver >> 32,
            0x0a0 => self.queue_device as u32 as u64,
            0x0a4 => self.queue_device >> 32,
            0x0fc => 0,
            0x100..=0x107 => return read_config_u64(self.capacity_sectors(), offset - 0x100, size),
            _ => 0,
        };
        mask_read(val, size)
    }

    pub fn write(&mut self, mem: &mut PhysicalMemory, offset: u64, value: u64, _size: u8) -> bool {
        match offset {
            0x014 => self.device_features_sel = value as u32,
            0x020 => {
                let _ = value;
            }
            0x024 => self.driver_features_sel = value as u32,
            0x030 => self.queue_sel = value as u32,
            0x038 => self.queue_num = (value as u16).min(QUEUE_NUM_MAX),
            0x044 => {
                self.queue_ready = value & 1 != 0;
                if !self.queue_ready {
                    self.last_avail_idx = 0;
                }
            }
            0x050 => return self.process_queue(mem),
            0x060 => self.interrupt_status = value as u32,
            0x064 => self.interrupt_status &= !(value as u32),
            0x070 => {
                self.status = value as u32;
                if value == 0 {
                    self.reset_queue();
                }
            }
            0x080 => self.queue_desc = (self.queue_desc & !0xffff_ffff) | (value as u32 as u64),
            0x084 => {
                self.queue_desc = (self.queue_desc & 0xffff_ffff) | ((value as u32 as u64) << 32)
            }
            0x090 => self.queue_driver = (self.queue_driver & !0xffff_ffff) | (value as u32 as u64),
            0x094 => {
                self.queue_driver =
                    (self.queue_driver & 0xffff_ffff) | ((value as u32 as u64) << 32)
            }
            0x0a0 => self.queue_device = (self.queue_device & !0xffff_ffff) | (value as u32 as u64),
            0x0a4 => {
                self.queue_device =
                    (self.queue_device & 0xffff_ffff) | ((value as u32 as u64) << 32)
            }
            _ => {}
        }
        false
    }

    fn selected_device_features(&self) -> u64 {
        let features = VIRTIO_F_VERSION_1;
        match self.device_features_sel {
            0 => features & 0xffff_ffff,
            1 => features >> 32,
            _ => 0,
        }
    }

    fn capacity_sectors(&self) -> u64 {
        self.image.len().div_ceil(SECTOR_SIZE) as u64
    }

    fn reset_queue(&mut self) {
        self.queue_ready = false;
        self.queue_num = 0;
        self.queue_desc = 0;
        self.queue_driver = 0;
        self.queue_device = 0;
        self.last_avail_idx = 0;
        self.interrupt_status = 0;
    }

    fn process_queue(&mut self, mem: &mut PhysicalMemory) -> bool {
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

    fn handle_request(&self, mem: &mut PhysicalMemory, head: u16) -> u32 {
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
                write_bytes(mem, data_desc.addr, data_desc.len, b"webboxvm-iso\0")
            }
            VIRTIO_BLK_T_FLUSH => (VIRTIO_BLK_S_OK, 0),
            VIRTIO_BLK_T_OUT => (VIRTIO_BLK_S_IOERR, 0),
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

        let start = sector as usize * SECTOR_SIZE;
        let len = desc.len as usize;
        for i in 0..len {
            let byte = self.image.get(start + i).copied().unwrap_or(0);
            let _ = mem.write(desc.addr + i as u64, 1, byte as u64);
        }
        (VIRTIO_BLK_S_OK, desc.len)
    }

    fn read_desc(&self, mem: &PhysicalMemory, index: u16) -> Option<Descriptor> {
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

    fn next_desc(&self, mem: &PhysicalMemory, desc: Descriptor) -> Option<Descriptor> {
        if desc.flags & VIRTQ_DESC_F_NEXT == 0 {
            return None;
        }
        self.read_desc(mem, desc.next)
    }

    fn push_used(&mut self, mem: &mut PhysicalMemory, id: u32, len: u32) {
        let used_idx = mem.read(self.queue_device + 2, 2).unwrap_or(0) as u16;
        let slot = used_idx % self.queue_num;
        let elem = self.queue_device + 4 + slot as u64 * 8;
        let _ = mem.write(elem, 4, id as u64);
        let _ = mem.write(elem + 4, 4, len as u64);
        let _ = mem.write(self.queue_device + 2, 2, used_idx.wrapping_add(1) as u64);
    }
}

impl Default for VirtioBlk {
    fn default() -> Self {
        Self::new()
    }
}

fn read_config_u64(value: u64, offset: u64, size: u8) -> Option<u64> {
    let bytes = value.to_le_bytes();
    let offset = offset as usize;
    match size {
        1 => Some(bytes[offset] as u64),
        2 => Some(u16::from_le_bytes([bytes[offset], bytes[offset + 1]]) as u64),
        4 => Some(u32::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]) as u64),
        8 if offset == 0 => Some(value),
        _ => None,
    }
}

fn mask_read(value: u64, size: u8) -> Option<u64> {
    match size {
        1 => Some(value & 0xff),
        2 => Some(value & 0xffff),
        4 => Some(value & 0xffff_ffff),
        8 => Some(value),
        _ => None,
    }
}

fn write_bytes(mem: &mut PhysicalMemory, addr: u64, len: u32, src: &[u8]) -> (u8, u32) {
    let count = (len as usize).min(src.len());
    for (i, byte) in src.iter().take(count).enumerate() {
        let _ = mem.write(addr + i as u64, 1, *byte as u64);
    }
    (VIRTIO_BLK_S_OK, count as u32)
}
