use super::super::*;
use crate::devices::virtio_blk::sparse_snapshot::SparseDiskSnapshot;
use crate::devices::virtio_blk::storage::{SparseDiskStorage, chunk_has_data};

fn parsed_snapshot(size_bytes: u64, chunks: &[(u64, &[u8])]) -> SparseDiskSnapshot {
    let mut snapshot = Vec::new();
    snapshot.extend_from_slice(SPARSE_DISK_SNAPSHOT_MAGIC);
    snapshot.extend_from_slice(&size_bytes.to_le_bytes());
    snapshot.extend_from_slice(&(SPARSE_DISK_CHUNK_SIZE as u32).to_le_bytes());
    snapshot.extend_from_slice(&(chunks.len() as u64).to_le_bytes());
    for (index, data) in chunks {
        snapshot.extend_from_slice(&index.to_le_bytes());
        let start = snapshot.len();
        snapshot.resize(start + SPARSE_DISK_CHUNK_SIZE, 0);
        snapshot[start..start + data.len()].copy_from_slice(data);
    }
    SparseDiskSnapshot::load(snapshot).unwrap()
}

#[test]
fn parsed_snapshot_is_read_directly_without_overlay_copy() {
    let base = parsed_snapshot(
        2 * SPARSE_DISK_CHUNK_SIZE as u64,
        &[(0, &[1, 2, 3]), (1, &[4, 5, 6])],
    );
    let disk = SparseDiskStorage::from_parsed_snapshot(base, b"disk\0");
    let mut first = [0xff; 5];
    let mut second = [0xff; 4];

    assert!(disk.overlay.is_empty());
    assert_eq!(disk.generation, 1);
    assert_eq!(disk.read(0, &mut first), VIRTIO_BLK_S_OK);
    assert_eq!(first, [1, 2, 3, 0, 0]);
    assert_eq!(
        disk.read(SPARSE_DISK_CHUNK_SIZE as u64, &mut second),
        VIRTIO_BLK_S_OK
    );
    assert_eq!(second, [4, 5, 6, 0]);
    assert_eq!(disk.allocated_bytes(), 2 * SPARSE_DISK_CHUNK_SIZE as u64);
}

#[test]
fn partial_overlay_write_preserves_untouched_base_bytes() {
    let base = parsed_snapshot(SPARSE_DISK_CHUNK_SIZE as u64, &[(0, &[1, 2, 3, 4, 5, 6])]);
    let base_probe = base.clone();
    let mut disk = SparseDiskStorage::from_parsed_snapshot(base, b"disk\0");
    let mut out = [0; 6];
    let mut original = [0; 6];

    assert_eq!(disk.write(2, &[9, 10]), VIRTIO_BLK_S_OK);
    assert_eq!(disk.read(0, &mut out), VIRTIO_BLK_S_OK);
    base_probe.read_at(0, &mut original).unwrap();

    assert_eq!(out, [1, 2, 9, 10, 5, 6]);
    assert_eq!(original, [1, 2, 3, 4, 5, 6]);
    assert_eq!(disk.overlay.len(), 1);
    assert_eq!(disk.allocated_bytes(), SPARSE_DISK_CHUNK_SIZE as u64);
}

#[test]
fn zero_overlay_shadows_base_and_removes_effective_allocation() {
    let base = parsed_snapshot(SPARSE_DISK_CHUNK_SIZE as u64, &[(0, &[1, 2, 3])]);
    let mut disk = SparseDiskStorage::from_parsed_snapshot(base, b"disk\0");
    let mut out = [0xff; 3];

    assert_eq!(disk.allocated_bytes(), SPARSE_DISK_CHUNK_SIZE as u64);
    assert_eq!(disk.write(0, &[0, 0, 0]), VIRTIO_BLK_S_OK);
    assert_eq!(disk.read(0, &mut out), VIRTIO_BLK_S_OK);

    assert_eq!(out, [0; 3]);
    assert_eq!(disk.allocated_bytes(), 0);
    assert_eq!(disk.overlay.len(), 1);
    assert!(!chunk_has_data(&disk.overlay[&0]));
    assert_eq!(disk.generation, 2);

    let snapshot = disk.snapshot();
    assert_eq!(snapshot.len(), SPARSE_DISK_SNAPSHOT_HEADER_LEN);
    assert_eq!(&snapshot[20..28], &0u64.to_le_bytes());
}

#[test]
fn base_and_overlay_snapshot_export_reload_roundtrips_effective_disk() {
    let base = parsed_snapshot(
        3 * SPARSE_DISK_CHUNK_SIZE as u64,
        &[(0, &[1, 2, 3, 4]), (1, &[5, 6, 7, 8])],
    );
    let mut disk = SparseDiskStorage::from_parsed_snapshot(base, b"disk\0");
    let chunk = SPARSE_DISK_CHUNK_SIZE as u64;

    assert_eq!(disk.write(1, &[9, 10]), VIRTIO_BLK_S_OK);
    assert_eq!(disk.write(chunk, &[0, 0, 0, 0]), VIRTIO_BLK_S_OK);
    assert_eq!(disk.write(2 * chunk + 7, &[11, 12]), VIRTIO_BLK_S_OK);

    let snapshot = disk.snapshot();
    let restored = SparseDiskStorage::from_snapshot(&snapshot, b"disk\0").unwrap();
    let mut first = [0; 4];
    let mut second = [0xff; 4];
    let mut third = [0; 2];

    assert_eq!(restored.read(0, &mut first), VIRTIO_BLK_S_OK);
    assert_eq!(restored.read(chunk, &mut second), VIRTIO_BLK_S_OK);
    assert_eq!(restored.read(2 * chunk + 7, &mut third), VIRTIO_BLK_S_OK);
    assert_eq!(first, [1, 9, 10, 4]);
    assert_eq!(second, [0; 4]);
    assert_eq!(third, [11, 12]);
    assert_eq!(
        restored.allocated_bytes(),
        2 * SPARSE_DISK_CHUNK_SIZE as u64
    );
    assert_eq!(restored.snapshot(), snapshot);
}

#[test]
fn attaching_parsed_snapshot_resets_virtqueue_state() {
    let mut device = VirtioBlk::writable_sparse(SPARSE_DISK_CHUNK_SIZE as u64, b"webboxvm-disk\0");
    device.queue_ready = true;
    device.queue_num = 16;
    device.queue_desc = 1;
    device.queue_driver = 2;
    device.queue_device = 3;
    device.last_avail_idx = 4;
    device.interrupt_status = 1;

    device.set_sparse_disk_snapshot(parsed_snapshot(SPARSE_DISK_CHUNK_SIZE as u64, &[]));

    assert!(!device.queue_ready);
    assert_eq!(device.queue_num, 0);
    assert_eq!(device.queue_desc, 0);
    assert_eq!(device.queue_driver, 0);
    assert_eq!(device.queue_device, 0);
    assert_eq!(device.last_avail_idx, 0);
    assert_eq!(device.interrupt_status, 0);
    assert_eq!(device.storage_generation(), 1);
}
