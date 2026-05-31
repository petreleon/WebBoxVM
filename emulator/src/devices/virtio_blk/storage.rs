mod io;
mod snapshot;

use super::*;
use io::read_image;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub(super) enum BlockStorage {
    ReadOnlyImage { image: Vec<u8>, id: &'static [u8] },
    SparseDisk(SparseDiskStorage),
}

impl BlockStorage {
    pub(super) fn capacity_sectors(&self) -> u64 {
        self.capacity_bytes().div_ceil(SECTOR_SIZE as u64)
    }

    pub(super) fn capacity_bytes(&self) -> u64 {
        match self {
            Self::ReadOnlyImage { image, .. } => image.len() as u64,
            Self::SparseDisk(disk) => disk.size_bytes,
        }
    }

    pub(super) fn id(&self) -> &'static [u8] {
        match self {
            Self::ReadOnlyImage { id, .. } => id,
            Self::SparseDisk(disk) => disk.id,
        }
    }

    pub(super) fn feature_bits(&self) -> u64 {
        match self {
            Self::ReadOnlyImage { .. } => VIRTIO_BLK_F_RO,
            Self::SparseDisk(_) => 0,
        }
    }

    pub(super) fn allocated_bytes(&self) -> u64 {
        match self {
            Self::ReadOnlyImage { image, .. } => image.len() as u64,
            Self::SparseDisk(disk) => disk.allocated_bytes(),
        }
    }

    pub(super) fn generation(&self) -> u64 {
        match self {
            Self::ReadOnlyImage { .. } => 0,
            Self::SparseDisk(disk) => disk.generation,
        }
    }

    pub(super) fn read(&self, offset: u64, dst: &mut [u8]) -> u8 {
        let Some(end) = offset.checked_add(dst.len() as u64) else {
            return VIRTIO_BLK_S_IOERR;
        };
        let capacity_bytes = self.capacity_sectors() * SECTOR_SIZE as u64;
        if end > capacity_bytes {
            return VIRTIO_BLK_S_IOERR;
        }

        match self {
            Self::ReadOnlyImage { image, .. } => read_image(image, offset, dst),
            Self::SparseDisk(disk) => disk.read(offset, dst),
        }
    }

    pub(super) fn write(&mut self, offset: u64, src: &[u8]) -> u8 {
        match self {
            Self::ReadOnlyImage { .. } => VIRTIO_BLK_S_IOERR,
            Self::SparseDisk(disk) => disk.write(offset, src),
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct SparseDiskStorage {
    pub(super) size_bytes: u64,
    pub(super) id: &'static [u8],
    pub(super) chunks: HashMap<u64, Box<[u8; SPARSE_DISK_CHUNK_SIZE]>>,
    pub(super) generation: u64,
}

impl SparseDiskStorage {
    pub(super) fn new(size_bytes: u64, id: &'static [u8]) -> Self {
        Self {
            size_bytes,
            id,
            chunks: HashMap::new(),
            generation: 0,
        }
    }

    pub(super) fn read(&self, offset: u64, dst: &mut [u8]) -> u8 {
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

    pub(super) fn write(&mut self, offset: u64, src: &[u8]) -> u8 {
        if !self.range_in_disk(offset, src.len()) {
            return VIRTIO_BLK_S_IOERR;
        }
        self.write_in_range(offset, src);
        if !src.is_empty() {
            self.generation = self.generation.wrapping_add(1);
        }
        VIRTIO_BLK_S_OK
    }

    pub(super) fn allocated_bytes(&self) -> u64 {
        self.chunks.len() as u64 * SPARSE_DISK_CHUNK_SIZE as u64
    }

    fn range_in_disk(&self, offset: u64, len: usize) -> bool {
        offset
            .checked_add(len as u64)
            .is_some_and(|end| end <= self.size_bytes)
    }
}
