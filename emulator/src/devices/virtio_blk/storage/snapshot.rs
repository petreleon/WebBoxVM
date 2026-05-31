use super::*;

impl BlockStorage {
    pub(in crate::devices::virtio_blk) fn snapshot(&self) -> Result<Vec<u8>, String> {
        match self {
            Self::ReadOnlyImage { .. } => Err("read-only block media cannot be snapshotted".into()),
            Self::SparseDisk(disk) => Ok(disk.snapshot()),
        }
    }

    pub(in crate::devices::virtio_blk) fn restore(
        &mut self,
        snapshot: &[u8],
    ) -> Result<(), String> {
        match self {
            Self::ReadOnlyImage { .. } => Err("read-only block media cannot be restored".into()),
            Self::SparseDisk(disk) => {
                *disk = SparseDiskStorage::from_snapshot(snapshot, disk.id)?;
                Ok(())
            }
        }
    }
}

impl SparseDiskStorage {
    pub(in crate::devices::virtio_blk) fn snapshot(&self) -> Vec<u8> {
        let mut chunk_indexes: Vec<u64> = self
            .chunks
            .iter()
            .filter_map(|(index, chunk)| chunk_has_data(chunk).then_some(*index))
            .collect();
        chunk_indexes.sort_unstable();

        let mut snapshot = Vec::with_capacity(
            SPARSE_DISK_SNAPSHOT_HEADER_LEN + chunk_indexes.len() * SPARSE_DISK_SNAPSHOT_ENTRY_LEN,
        );
        snapshot.extend_from_slice(SPARSE_DISK_SNAPSHOT_MAGIC);
        snapshot.extend_from_slice(&self.size_bytes.to_le_bytes());
        snapshot.extend_from_slice(&(SPARSE_DISK_CHUNK_SIZE as u32).to_le_bytes());
        snapshot.extend_from_slice(&(chunk_indexes.len() as u64).to_le_bytes());

        for index in chunk_indexes {
            snapshot.extend_from_slice(&index.to_le_bytes());
            snapshot.extend_from_slice(&self.chunks[&index][..]);
        }

        snapshot
    }

    pub(in crate::devices::virtio_blk) fn from_snapshot(
        snapshot: &[u8],
        id: &'static [u8],
    ) -> Result<Self, String> {
        validate_header(snapshot)?;
        let size_bytes = read_le_u64(snapshot, 8)?;
        let chunk_count = usize::try_from(read_le_u64(snapshot, 20)?)
            .map_err(|_| "persistent disk chunk count is too large".to_string())?;
        validate_len(snapshot, chunk_count)?;

        let mut disk = Self::new(size_bytes, id);
        let mut offset = SPARSE_DISK_SNAPSHOT_HEADER_LEN;
        for _ in 0..chunk_count {
            let chunk_index = read_le_u64(snapshot, offset)?;
            offset += 8;
            validate_chunk(size_bytes, chunk_index)?;

            let mut chunk = Box::new([0; SPARSE_DISK_CHUNK_SIZE]);
            chunk.copy_from_slice(&snapshot[offset..offset + SPARSE_DISK_CHUNK_SIZE]);
            offset += SPARSE_DISK_CHUNK_SIZE;

            if chunk_has_data(&chunk) {
                disk.chunks.insert(chunk_index, chunk);
            }
        }
        disk.generation = 1;
        Ok(disk)
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
        return Err(format!(
            "persistent disk chunk size mismatch: got {chunk_size}, expected {SPARSE_DISK_CHUNK_SIZE}"
        ));
    }
    Ok(())
}

fn validate_len(snapshot: &[u8], chunk_count: usize) -> Result<(), String> {
    let body_len = chunk_count
        .checked_mul(SPARSE_DISK_SNAPSHOT_ENTRY_LEN)
        .ok_or_else(|| "persistent disk snapshot is too large".to_string())?;
    let expected_len = SPARSE_DISK_SNAPSHOT_HEADER_LEN
        .checked_add(body_len)
        .ok_or_else(|| "persistent disk snapshot is too large".to_string())?;
    if snapshot.len() != expected_len {
        return Err("persistent disk snapshot length does not match header".to_string());
    }
    Ok(())
}

fn validate_chunk(size_bytes: u64, chunk_index: u64) -> Result<(), String> {
    let chunk_start = chunk_index
        .checked_mul(SPARSE_DISK_CHUNK_SIZE as u64)
        .ok_or_else(|| "persistent disk chunk index overflows".to_string())?;
    if chunk_start >= size_bytes {
        return Err("persistent disk chunk lies beyond disk capacity".to_string());
    }
    Ok(())
}

pub(super) fn chunk_has_data(chunk: &[u8; SPARSE_DISK_CHUNK_SIZE]) -> bool {
    chunk.iter().any(|byte| *byte != 0)
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
