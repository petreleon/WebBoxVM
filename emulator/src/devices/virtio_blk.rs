//! Minimal VirtIO-MMIO block device.
//!
//! This implements the subset Linux needs for installer media and a target disk:
//! feature negotiation, one split virtqueue, reads, read-only media, sparse
//! writes, and compact sparse-disk snapshots.

mod mmio;
mod queue;
pub mod sparse_snapshot;
mod storage;
#[cfg(test)]
mod tests;

use crate::memory::PhysicalMemory;
use storage::BlockStorage;

pub(super) const VIRTIO_MMIO_MAGIC: u64 = 0x7472_6976;
pub(super) const VIRTIO_MMIO_VERSION_2: u64 = 2;
pub(super) const VIRTIO_DEVICE_ID_BLOCK: u64 = 2;
pub(super) const VIRTIO_VENDOR_WEBBOXVM: u64 = 0x5742_564d;

pub(super) const VIRTIO_BLK_F_RO: u64 = 1 << 5;
pub(super) const VIRTIO_F_VERSION_1: u64 = 1 << 32;

pub(super) const VIRTQ_DESC_F_NEXT: u16 = 1;
pub(super) const VIRTQ_DESC_F_WRITE: u16 = 2;

pub(super) const VIRTIO_BLK_T_IN: u32 = 0;
pub(super) const VIRTIO_BLK_T_OUT: u32 = 1;
pub(super) const VIRTIO_BLK_T_FLUSH: u32 = 4;
pub(super) const VIRTIO_BLK_T_GET_ID: u32 = 8;

pub(super) const VIRTIO_BLK_S_OK: u8 = 0;
pub(super) const VIRTIO_BLK_S_IOERR: u8 = 1;
pub(super) const VIRTIO_BLK_S_UNSUPP: u8 = 2;

pub(super) const SECTOR_SIZE: usize = 512;
pub(super) const QUEUE_NUM_MAX: u16 = 64;
pub(super) const SPARSE_DISK_CHUNK_SIZE: usize = 64 * 1024;
pub(super) const SPARSE_DISK_SNAPSHOT_MAGIC: &[u8; 8] = b"WBDISK01";
pub(super) const SPARSE_DISK_SNAPSHOT_HEADER_LEN: usize = 28;
pub(super) const SPARSE_DISK_SNAPSHOT_ENTRY_LEN: usize = 8 + SPARSE_DISK_CHUNK_SIZE;
pub const DEFAULT_SPARSE_DISK_SIZE: u64 = 4 * 1024 * 1024 * 1024;

#[derive(Clone, Copy, Debug)]
pub(super) struct Descriptor {
    pub(in crate::devices::virtio_blk) addr: u64,
    pub(in crate::devices::virtio_blk) len: u32,
    pub(in crate::devices::virtio_blk) flags: u16,
    pub(in crate::devices::virtio_blk) next: u16,
}

#[derive(Debug, Clone)]
pub struct VirtioBlk {
    pub(in crate::devices::virtio_blk) storage: BlockStorage,
    pub(in crate::devices::virtio_blk) device_features_sel: u32,
    pub(in crate::devices::virtio_blk) driver_features_sel: u32,
    pub(in crate::devices::virtio_blk) queue_sel: u32,
    pub(in crate::devices::virtio_blk) queue_num: u16,
    pub(in crate::devices::virtio_blk) queue_ready: bool,
    pub(in crate::devices::virtio_blk) queue_desc: u64,
    pub(in crate::devices::virtio_blk) queue_driver: u64,
    pub(in crate::devices::virtio_blk) queue_device: u64,
    pub(in crate::devices::virtio_blk) last_avail_idx: u16,
    pub(in crate::devices::virtio_blk) interrupt_status: u32,
    pub(in crate::devices::virtio_blk) status: u32,
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
            storage: BlockStorage::SparseDisk(storage::SparseDiskStorage::new(size_bytes, id)),
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
        self.storage = BlockStorage::SparseDisk(storage::SparseDiskStorage::new(
            size_bytes,
            b"webboxvm-disk\0",
        ));
        self.reset_queue();
    }
}

impl Default for VirtioBlk {
    fn default() -> Self {
        Self::new()
    }
}
