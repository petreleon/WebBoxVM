use super::super::*;
use crate::devices::virtio_blk::storage::SparseDiskStorage;

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
    let storage = BlockStorage::SparseDisk(SparseDiskStorage::new(SECTOR_SIZE as u64, b"disk\0"));
    let mut bytes = [0u8; SECTOR_SIZE + 1];

    assert_eq!(storage.read(0, &mut bytes), VIRTIO_BLK_S_IOERR);
}

#[test]
fn sparse_disk_snapshot_roundtrips_nonzero_chunks() {
    let mut original = SparseDiskStorage::new(3 * SPARSE_DISK_CHUNK_SIZE as u64, b"disk\0");
    let offset = SPARSE_DISK_CHUNK_SIZE as u64 + 9;
    let mut out = [0u8; 4];

    assert_eq!(original.write(offset, &[7, 8, 9, 10]), VIRTIO_BLK_S_OK);
    let snapshot = original.snapshot();
    let restored = SparseDiskStorage::from_snapshot(&snapshot, b"disk\0").unwrap();

    assert_eq!(restored.size_bytes, original.size_bytes);
    assert_eq!(restored.allocated_bytes(), SPARSE_DISK_CHUNK_SIZE as u64);
    assert_eq!(restored.read(offset, &mut out), VIRTIO_BLK_S_OK);
    assert_eq!(out, [7, 8, 9, 10]);
}

#[test]
fn sparse_disk_snapshot_rejects_out_of_range_chunks() {
    let mut snapshot = SparseDiskStorage::new(SPARSE_DISK_CHUNK_SIZE as u64, b"disk\0").snapshot();
    snapshot[20..28].copy_from_slice(&1u64.to_le_bytes());
    snapshot.extend_from_slice(&2u64.to_le_bytes());
    snapshot.extend_from_slice(&[1; SPARSE_DISK_CHUNK_SIZE]);

    let err = SparseDiskStorage::from_snapshot(&snapshot, b"disk\0").unwrap_err();
    assert!(err.contains("beyond disk capacity"));
}
