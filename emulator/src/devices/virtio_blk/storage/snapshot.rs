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
        let mut chunk_indexes = Vec::new();
        if let Some(base) = &self.base {
            chunk_indexes
                .extend(base.chunks().filter_map(|(index, _)| {
                    (!self.overlay.contains_key(&index)).then_some(index)
                }));
        }
        chunk_indexes.extend(
            self.overlay
                .iter()
                .filter_map(|(index, chunk)| chunk_has_data(chunk).then_some(*index)),
        );
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
            let chunk = self
                .overlay
                .get(&index)
                .map(Box::as_ref)
                .or_else(|| self.base_chunk(index))
                .expect("effective sparse chunk must exist");
            snapshot.extend_from_slice(chunk);
        }

        snapshot
    }

    pub(in crate::devices::virtio_blk) fn from_snapshot(
        snapshot: &[u8],
        id: &'static [u8],
    ) -> Result<Self, String> {
        let snapshot = SparseDiskSnapshot::load(snapshot.to_vec())?;
        Ok(Self::from_parsed_snapshot(snapshot, id))
    }
}
