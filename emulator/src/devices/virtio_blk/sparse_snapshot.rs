use super::*;
use std::collections::HashMap;
use std::ops::Range;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct SparseDiskSnapshot {
    data: Arc<Vec<u8>>,
    size_bytes: u64,
    chunks: Arc<[SparseChunk]>,
    positions: Arc<HashMap<u64, usize>>,
}

#[derive(Clone, Debug)]
struct SparseChunk {
    index: u64,
    data: Range<usize>,
}

impl SparseDiskSnapshot {
    pub fn load(snapshot: Vec<u8>) -> Result<Self, String> {
        validate_header(&snapshot)?;
        let size_bytes = read_le_u64(&snapshot, 8)?;
        let chunk_count = usize::try_from(read_le_u64(&snapshot, 20)?)
            .map_err(|_| "persistent disk chunk count is too large".to_string())?;
        validate_len(&snapshot, chunk_count)?;

        let mut chunks = Vec::with_capacity(chunk_count);
        let mut offset = SPARSE_DISK_SNAPSHOT_HEADER_LEN;
        for _ in 0..chunk_count {
            let index = read_le_u64(&snapshot, offset)?;
            offset += 8;
            validate_chunk(size_bytes, index)?;
            chunks.push(SparseChunk {
                index,
                data: offset..offset + SPARSE_DISK_CHUNK_SIZE,
            });
            offset += SPARSE_DISK_CHUNK_SIZE;
        }
        chunks.sort_by_key(|chunk| chunk.index);
        reject_duplicates(&chunks)?;
        let positions = chunks
            .iter()
            .enumerate()
            .map(|(position, chunk)| (chunk.index, position))
            .collect();

        Ok(Self {
            data: Arc::new(snapshot),
            size_bytes,
            chunks: Arc::from(chunks),
            positions: Arc::new(positions),
        })
    }

    pub fn size_bytes(&self) -> u64 {
        self.size_bytes
    }

    pub fn read_at(&self, offset: u64, dst: &mut [u8]) -> Result<(), String> {
        if offset
            .checked_add(dst.len() as u64)
            .is_none_or(|end| end > self.size_bytes)
        {
            return Err("persistent disk read is outside disk capacity".to_string());
        }

        let mut done = 0usize;
        while done < dst.len() {
            let current = offset + done as u64;
            let index = current / SPARSE_DISK_CHUNK_SIZE as u64;
            let in_chunk = (current % SPARSE_DISK_CHUNK_SIZE as u64) as usize;
            let count = (dst.len() - done).min(SPARSE_DISK_CHUNK_SIZE - in_chunk);
            if let Some(chunk) = self.chunk_data(index) {
                dst[done..done + count].copy_from_slice(&chunk[in_chunk..in_chunk + count]);
            } else {
                dst[done..done + count].fill(0);
            }
            done += count;
        }
        Ok(())
    }

    pub(in crate::devices::virtio_blk) fn chunk_data(
        &self,
        index: u64,
    ) -> Option<&[u8; SPARSE_DISK_CHUNK_SIZE]> {
        self.find_chunk(index).map(|chunk| chunk.data(&self.data))
    }

    pub(in crate::devices::virtio_blk) fn chunks(
        &self,
    ) -> impl Iterator<Item = (u64, &[u8; SPARSE_DISK_CHUNK_SIZE])> {
        self.chunks
            .iter()
            .map(|chunk| (chunk.index, chunk.data(&self.data)))
    }

    pub(in crate::devices::virtio_blk) fn chunk_count(&self) -> usize {
        self.chunks.len()
    }

    #[cfg(test)]
    pub(in crate::devices::virtio_blk) fn shares_backing(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.data, &other.data)
    }

    fn find_chunk(&self, index: u64) -> Option<&SparseChunk> {
        self.positions
            .get(&index)
            .map(|position| &self.chunks[*position])
    }
}

impl SparseChunk {
    fn data<'a>(&self, snapshot: &'a [u8]) -> &'a [u8; SPARSE_DISK_CHUNK_SIZE] {
        snapshot[self.data.clone()]
            .try_into()
            .expect("validated sparse snapshot chunk length")
    }
}

fn validate_header(snapshot: &[u8]) -> Result<(), String> {
    if snapshot.len() < SPARSE_DISK_SNAPSHOT_HEADER_LEN {
        return Err("persistent disk snapshot is too small".to_string());
    }
    if &snapshot[..8] != SPARSE_DISK_SNAPSHOT_MAGIC {
        return Err("persistent disk snapshot has an invalid magic".to_string());
    }
    let chunk_size = read_le_u32(snapshot, 16)? as usize;
    if chunk_size != SPARSE_DISK_CHUNK_SIZE {
        return Err("persistent disk chunk size mismatch".to_string());
    }
    Ok(())
}

fn validate_len(snapshot: &[u8], chunk_count: usize) -> Result<(), String> {
    let body_len = chunk_count
        .checked_mul(SPARSE_DISK_SNAPSHOT_ENTRY_LEN)
        .ok_or_else(|| "persistent disk snapshot is too large".to_string())?;
    let expected = SPARSE_DISK_SNAPSHOT_HEADER_LEN
        .checked_add(body_len)
        .ok_or_else(|| "persistent disk snapshot is too large".to_string())?;
    if snapshot.len() != expected {
        return Err("persistent disk snapshot length does not match header".to_string());
    }
    Ok(())
}

fn validate_chunk(size_bytes: u64, chunk_index: u64) -> Result<(), String> {
    let start = chunk_index
        .checked_mul(SPARSE_DISK_CHUNK_SIZE as u64)
        .ok_or_else(|| "persistent disk chunk index overflows".to_string())?;
    if start >= size_bytes {
        return Err("persistent disk chunk lies beyond disk capacity".to_string());
    }
    Ok(())
}

fn reject_duplicates(chunks: &[SparseChunk]) -> Result<(), String> {
    if chunks.windows(2).any(|pair| pair[0].index == pair[1].index) {
        return Err("persistent disk snapshot contains duplicate chunks".to_string());
    }
    Ok(())
}

fn read_le_u32(bytes: &[u8], offset: usize) -> Result<u32, String> {
    let data = bytes
        .get(offset..offset + 4)
        .ok_or_else(|| "persistent disk snapshot is truncated".to_string())?;
    Ok(u32::from_le_bytes(data.try_into().unwrap()))
}

fn read_le_u64(bytes: &[u8], offset: usize) -> Result<u64, String> {
    let data = bytes
        .get(offset..offset + 8)
        .ok_or_else(|| "persistent disk snapshot is truncated".to_string())?;
    Ok(u64::from_le_bytes(data.try_into().unwrap()))
}
