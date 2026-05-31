//! Minimal VirtIO-MMIO block device.
//!
//! This implements the subset Linux needs for installer media and a target disk:
//! feature negotiation, one split virtqueue, reads, read-only media, and sparse
//! writes.

use crate::memory::PhysicalMemory;
use std::collections::HashMap;

const VIRTIO_MMIO_MAGIC: u64 = 0x7472_6976;
const VIRTIO_MMIO_VERSION_2: u64 = 2;
const VIRTIO_DEVICE_ID_BLOCK: u64 = 2;
const VIRTIO_VENDOR_WEBBOXVM: u64 = 0x5742_564d;

const VIRTIO_BLK_F_RO: u64 = 1 << 5;
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
const SPARSE_DISK_CHUNK_SIZE: usize = 64 * 1024;
pub const DEFAULT_SPARSE_DISK_SIZE: u64 = 4 * 1024 * 1024 * 1024;

#[derive(Clone, Copy, Debug)]
struct Descriptor {
    addr: u64,
    len: u32,
    flags: u16,
    next: u16,
}

#[derive(Debug, Clone)]
pub struct VirtioBlk {
    storage: BlockStorage,
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
        Self::read_only_image(Vec::new(), b"webboxvm-iso\0")
    }

    pub fn read_only_image(image: Vec<u8>, id: &'static [u8]) -> Self {
        Self {
            storage: BlockStorage::ReadOnlyImage { image, id },
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

    pub fn writable_sparse(size_bytes: u64, id: &'static [u8]) -> Self {
        Self {
            storage: BlockStorage::SparseDisk(SparseDiskStorage::new(size_bytes, id)),
            ..Self::new()
        }
    }

    pub fn set_image(&mut self, image: &[u8]) {
        self.storage = BlockStorage::ReadOnlyImage {
            image: image.to_vec(),
            id: b"webboxvm-iso\0",
        };
    }

    pub fn set_image_owned(&mut self, image: Vec<u8>) {
        self.storage = BlockStorage::ReadOnlyImage {
            image,
            id: b"webboxvm-iso\0",
        };
    }

    pub fn set_sparse_disk(&mut self, size_bytes: u64) {
        self.storage =
            BlockStorage::SparseDisk(SparseDiskStorage::new(size_bytes, b"webboxvm-disk\0"));
        self.reset_queue();
    }

    pub fn allocated_storage_bytes(&self) -> u64 {
        self.storage.allocated_bytes()
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
        let features = VIRTIO_F_VERSION_1 | self.storage.feature_bits();
        match self.device_features_sel {
            0 => features & 0xffff_ffff,
            1 => features >> 32,
            _ => 0,
        }
    }

    fn capacity_sectors(&self) -> u64 {
        self.storage.capacity_sectors()
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
    let len = match size {
        1 | 2 | 4 | 8 => Some(size as usize),
        _ => None,
    }?;
    let end = offset.checked_add(len)?;
    if end > bytes.len() {
        return None;
    }

    let mut out = [0u8; 8];
    out[..len].copy_from_slice(&bytes[offset..end]);
    Some(u64::from_le_bytes(out))
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
    if count == 0 {
        return (VIRTIO_BLK_S_OK, 0);
    }
    if mem.write_bytes(addr, &src[..count]).is_none() {
        return (VIRTIO_BLK_S_IOERR, 0);
    }
    (VIRTIO_BLK_S_OK, count as u32)
}

#[derive(Debug, Clone)]
enum BlockStorage {
    ReadOnlyImage { image: Vec<u8>, id: &'static [u8] },
    SparseDisk(SparseDiskStorage),
}

impl BlockStorage {
    fn capacity_sectors(&self) -> u64 {
        self.capacity_bytes().div_ceil(SECTOR_SIZE as u64)
    }

    fn capacity_bytes(&self) -> u64 {
        match self {
            Self::ReadOnlyImage { image, .. } => image.len() as u64,
            Self::SparseDisk(disk) => disk.size_bytes,
        }
    }

    fn id(&self) -> &'static [u8] {
        match self {
            Self::ReadOnlyImage { id, .. } => id,
            Self::SparseDisk(disk) => disk.id,
        }
    }

    fn feature_bits(&self) -> u64 {
        match self {
            Self::ReadOnlyImage { .. } => VIRTIO_BLK_F_RO,
            Self::SparseDisk(_) => 0,
        }
    }

    fn allocated_bytes(&self) -> u64 {
        match self {
            Self::ReadOnlyImage { image, .. } => image.len() as u64,
            Self::SparseDisk(disk) => disk.allocated_bytes(),
        }
    }

    fn read(&self, offset: u64, dst: &mut [u8]) -> u8 {
        let Some(end) = offset.checked_add(dst.len() as u64) else {
            return VIRTIO_BLK_S_IOERR;
        };
        let capacity_bytes = self.capacity_sectors() * SECTOR_SIZE as u64;
        if end > capacity_bytes {
            return VIRTIO_BLK_S_IOERR;
        }

        match self {
            Self::ReadOnlyImage { image, .. } => {
                if offset < image.len() as u64 {
                    let available = ((image.len() as u64 - offset) as usize).min(dst.len());
                    let start = offset as usize;
                    dst[..available].copy_from_slice(&image[start..start + available]);
                    dst[available..].fill(0);
                } else {
                    dst.fill(0);
                }
                VIRTIO_BLK_S_OK
            }
            Self::SparseDisk(disk) => disk.read(offset, dst),
        }
    }

    fn write(&mut self, offset: u64, src: &[u8]) -> u8 {
        match self {
            Self::ReadOnlyImage { .. } => VIRTIO_BLK_S_IOERR,
            Self::SparseDisk(disk) => disk.write(offset, src),
        }
    }
}

#[derive(Debug, Clone)]
struct SparseDiskStorage {
    size_bytes: u64,
    id: &'static [u8],
    chunks: HashMap<u64, Box<[u8; SPARSE_DISK_CHUNK_SIZE]>>,
}

impl SparseDiskStorage {
    fn new(size_bytes: u64, id: &'static [u8]) -> Self {
        Self {
            size_bytes,
            id,
            chunks: HashMap::new(),
        }
    }

    fn read(&self, offset: u64, dst: &mut [u8]) -> u8 {
        if !self.range_in_disk(offset, dst.len()) {
            return VIRTIO_BLK_S_IOERR;
        }

        let mut done = 0usize;
        while done < dst.len() {
            let current = offset + done as u64;
            let chunk_index = current / SPARSE_DISK_CHUNK_SIZE as u64;
            let chunk_offset = (current % SPARSE_DISK_CHUNK_SIZE as u64) as usize;
            let count = (dst.len() - done).min(SPARSE_DISK_CHUNK_SIZE - chunk_offset);

            if let Some(chunk) = self.chunks.get(&chunk_index) {
                dst[done..done + count].copy_from_slice(&chunk[chunk_offset..chunk_offset + count]);
            } else {
                dst[done..done + count].fill(0);
            }
            done += count;
        }

        VIRTIO_BLK_S_OK
    }

    fn write(&mut self, offset: u64, src: &[u8]) -> u8 {
        if !self.range_in_disk(offset, src.len()) {
            return VIRTIO_BLK_S_IOERR;
        }

        let mut done = 0usize;
        while done < src.len() {
            let current = offset + done as u64;
            let chunk_index = current / SPARSE_DISK_CHUNK_SIZE as u64;
            let chunk_offset = (current % SPARSE_DISK_CHUNK_SIZE as u64) as usize;
            let count = (src.len() - done).min(SPARSE_DISK_CHUNK_SIZE - chunk_offset);
            let chunk = self
                .chunks
                .entry(chunk_index)
                .or_insert_with(|| Box::new([0; SPARSE_DISK_CHUNK_SIZE]));

            chunk[chunk_offset..chunk_offset + count].copy_from_slice(&src[done..done + count]);
            done += count;
        }

        VIRTIO_BLK_S_OK
    }

    fn allocated_bytes(&self) -> u64 {
        self.chunks.len() as u64 * SPARSE_DISK_CHUNK_SIZE as u64
    }

    fn range_in_disk(&self, offset: u64, len: usize) -> bool {
        offset
            .checked_add(len as u64)
            .is_some_and(|end| end <= self.size_bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::RAM_BASE;

    const QUEUE_DESC: u64 = RAM_BASE + 0x1000;
    const QUEUE_DRIVER: u64 = RAM_BASE + 0x2000;
    const QUEUE_DEVICE: u64 = RAM_BASE + 0x3000;
    const REQ_ADDR: u64 = RAM_BASE + 0x4000;
    const DATA_ADDR: u64 = RAM_BASE + 0x5000;
    const STATUS_ADDR: u64 = RAM_BASE + 0x6000;

    fn configure_queue(device: &mut VirtioBlk, mem: &mut PhysicalMemory) {
        device.write(mem, 0x038, 8, 4);
        device.write(mem, 0x080, QUEUE_DESC as u32 as u64, 4);
        device.write(mem, 0x084, QUEUE_DESC >> 32, 4);
        device.write(mem, 0x090, QUEUE_DRIVER as u32 as u64, 4);
        device.write(mem, 0x094, QUEUE_DRIVER >> 32, 4);
        device.write(mem, 0x0a0, QUEUE_DEVICE as u32 as u64, 4);
        device.write(mem, 0x0a4, QUEUE_DEVICE >> 32, 4);
        device.write(mem, 0x044, 1, 4);
    }

    fn write_desc(
        mem: &mut PhysicalMemory,
        index: u16,
        addr: u64,
        len: u32,
        flags: u16,
        next: u16,
    ) {
        let base = QUEUE_DESC + index as u64 * 16;
        mem.write(base, 8, addr).unwrap();
        mem.write(base + 8, 4, len as u64).unwrap();
        mem.write(base + 12, 2, flags as u64).unwrap();
        mem.write(base + 14, 2, next as u64).unwrap();
    }

    fn submit_request(
        device: &mut VirtioBlk,
        mem: &mut PhysicalMemory,
        req_type: u32,
        sector: u64,
        data_len: u32,
        data_flags: u16,
        avail_idx: u16,
    ) -> u8 {
        mem.write(REQ_ADDR, 4, req_type as u64).unwrap();
        mem.write(REQ_ADDR + 4, 4, 0).unwrap();
        mem.write(REQ_ADDR + 8, 8, sector).unwrap();
        mem.write(STATUS_ADDR, 1, 0xff).unwrap();

        write_desc(mem, 0, REQ_ADDR, 16, VIRTQ_DESC_F_NEXT, 1);
        write_desc(
            mem,
            1,
            DATA_ADDR,
            data_len,
            data_flags | VIRTQ_DESC_F_NEXT,
            2,
        );
        write_desc(mem, 2, STATUS_ADDR, 1, VIRTQ_DESC_F_WRITE, 0);

        let ring_slot = avail_idx % 8;
        mem.write(QUEUE_DRIVER + 4 + ring_slot as u64 * 2, 2, 0)
            .unwrap();
        mem.write(QUEUE_DRIVER + 2, 2, avail_idx.wrapping_add(1) as u64)
            .unwrap();
        assert!(device.write(mem, 0x050, 0, 4));

        mem.read(STATUS_ADDR, 1).unwrap() as u8
    }

    #[test]
    fn sparse_disk_reads_zero_then_persists_writes() {
        let mut storage = BlockStorage::SparseDisk(SparseDiskStorage::new(
            2 * SPARSE_DISK_CHUNK_SIZE as u64,
            b"disk\0",
        ));
        let offset = SPARSE_DISK_CHUNK_SIZE as u64 - 2;
        let mut out = [0xff; 5];

        assert_eq!(storage.read(offset, &mut out), VIRTIO_BLK_S_OK);
        assert_eq!(out, [0; 5]);
        assert_eq!(storage.allocated_bytes(), 0);

        assert_eq!(storage.write(offset, &[1, 2, 3, 4, 5]), VIRTIO_BLK_S_OK);
        assert_eq!(storage.allocated_bytes(), 2 * SPARSE_DISK_CHUNK_SIZE as u64);
        assert_eq!(storage.read(offset, &mut out), VIRTIO_BLK_S_OK);
        assert_eq!(out, [1, 2, 3, 4, 5]);
    }

    #[test]
    fn read_only_image_rejects_writes_and_pads_last_sector() {
        let mut storage = BlockStorage::ReadOnlyImage {
            image: vec![1, 2, 3],
            id: b"iso\0",
        };
        let mut sector = [0xff; SECTOR_SIZE];

        assert_eq!(storage.read(0, &mut sector), VIRTIO_BLK_S_OK);
        assert_eq!(&sector[..5], &[1, 2, 3, 0, 0]);
        assert_eq!(storage.write(0, &[9]), VIRTIO_BLK_S_IOERR);
    }

    #[test]
    fn read_beyond_virtual_capacity_fails() {
        let storage =
            BlockStorage::SparseDisk(SparseDiskStorage::new(SECTOR_SIZE as u64, b"disk\0"));
        let mut bytes = [0u8; SECTOR_SIZE + 1];

        assert_eq!(storage.read(0, &mut bytes), VIRTIO_BLK_S_IOERR);
    }

    #[test]
    fn config_read_rejects_out_of_range_sizes() {
        let device = VirtioBlk::writable_sparse(SECTOR_SIZE as u64, b"disk\0");

        assert_eq!(device.read(0x106, 4), None);
        assert_eq!(device.read(0x107, 2), None);
        assert_eq!(device.read(0x100, 8), Some(1));
    }

    #[test]
    fn virtqueue_writes_to_sparse_disk_and_reads_back() {
        let mut mem = PhysicalMemory::new();
        let mut device = VirtioBlk::writable_sparse(SECTOR_SIZE as u64 * 8, b"disk\0");
        configure_queue(&mut device, &mut mem);

        mem.write_bytes(DATA_ADDR, b"hello").unwrap();
        assert_eq!(
            submit_request(&mut device, &mut mem, VIRTIO_BLK_T_OUT, 2, 5, 0, 0,),
            VIRTIO_BLK_S_OK
        );
        assert_eq!(
            device.allocated_storage_bytes(),
            SPARSE_DISK_CHUNK_SIZE as u64
        );

        mem.write_bytes(DATA_ADDR, &[0; 5]).unwrap();
        assert_eq!(
            submit_request(
                &mut device,
                &mut mem,
                VIRTIO_BLK_T_IN,
                2,
                5,
                VIRTQ_DESC_F_WRITE,
                1,
            ),
            VIRTIO_BLK_S_OK
        );

        let mut out = [0u8; 5];
        mem.read_bytes(DATA_ADDR, &mut out).unwrap();
        assert_eq!(&out, b"hello");
    }
}
