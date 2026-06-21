use super::super::*;
use crate::devices::virtio_blk::sparse_snapshot::SparseDiskSnapshot;

fn snapshot(size_bytes: u64, chunks: &[(u64, &[u8])]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(SPARSE_DISK_SNAPSHOT_MAGIC);
    out.extend_from_slice(&size_bytes.to_le_bytes());
    out.extend_from_slice(&(SPARSE_DISK_CHUNK_SIZE as u32).to_le_bytes());
    out.extend_from_slice(&(chunks.len() as u64).to_le_bytes());
    for (index, data) in chunks {
        let mut chunk = [0u8; SPARSE_DISK_CHUNK_SIZE];
        chunk[..data.len()].copy_from_slice(data);
        out.extend_from_slice(&index.to_le_bytes());
        out.extend_from_slice(&chunk);
    }
    out
}

#[test]
fn sparse_snapshot_reads_missing_chunks_as_zero() {
    let disk = SparseDiskSnapshot::load(snapshot(3 * SPARSE_DISK_CHUNK_SIZE as u64, &[])).unwrap();
    let mut out = [0xff; 8];

    disk.read_at(SPARSE_DISK_CHUNK_SIZE as u64 + 7, &mut out)
        .unwrap();

    assert_eq!(out, [0; 8]);
}

#[test]
fn sparse_snapshot_reads_across_chunk_boundary() {
    let first = vec![9; SPARSE_DISK_CHUNK_SIZE];
    let second = [1, 2, 3, 4];
    let disk = SparseDiskSnapshot::load(snapshot(
        2 * SPARSE_DISK_CHUNK_SIZE as u64,
        &[(0, &first), (1, &second)],
    ))
    .unwrap();
    let mut out = [0; 6];

    disk.read_at(SPARSE_DISK_CHUNK_SIZE as u64 - 2, &mut out)
        .unwrap();

    assert_eq!(out, [9, 9, 1, 2, 3, 4]);
}

#[test]
fn sparse_snapshot_rejects_duplicate_chunks() {
    let err = SparseDiskSnapshot::load(snapshot(
        SPARSE_DISK_CHUNK_SIZE as u64,
        &[(0, &[1]), (0, &[2])],
    ))
    .unwrap_err();

    assert!(err.contains("duplicate"));
}
