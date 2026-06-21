use super::partitions::{Partition, read_partitions};
use crate::devices::virtio_blk::sparse_snapshot::SparseDiskSnapshot;

const SECTOR: usize = 512;
const MBR_TABLE: usize = 446;
const GPT_SIGNATURE: &[u8; 8] = b"EFI PART";

fn snapshot_from_raw(raw: &[u8]) -> SparseDiskSnapshot {
    const CHUNK: usize = 64 * 1024;
    let chunks: Vec<_> = raw
        .chunks(CHUNK)
        .enumerate()
        .filter(|(_, chunk)| chunk.iter().any(|byte| *byte != 0))
        .collect();
    let mut snapshot = Vec::new();
    snapshot.extend_from_slice(b"WBDISK01");
    snapshot.extend_from_slice(&(raw.len() as u64).to_le_bytes());
    snapshot.extend_from_slice(&(CHUNK as u32).to_le_bytes());
    snapshot.extend_from_slice(&(chunks.len() as u64).to_le_bytes());
    for (index, data) in chunks {
        let mut chunk = [0u8; CHUNK];
        chunk[..data.len()].copy_from_slice(data);
        snapshot.extend_from_slice(&(index as u64).to_le_bytes());
        snapshot.extend_from_slice(&chunk);
    }
    SparseDiskSnapshot::load(snapshot).unwrap()
}

#[test]
fn reads_mbr_partitions() {
    let mut raw = vec![0; 128 * 1024];
    let entry = MBR_TABLE;
    raw[entry + 4] = 0x83;
    raw[entry + 8..entry + 12].copy_from_slice(&10u32.to_le_bytes());
    raw[entry + 12..entry + 16].copy_from_slice(&20u32.to_le_bytes());
    let disk = snapshot_from_raw(&raw);

    let partitions = read_partitions(&disk).unwrap();

    assert_eq!(
        partitions,
        vec![Partition {
            number: 1,
            start_lba: 10,
            end_lba: 29
        }]
    );
}

#[test]
fn reads_gpt_partitions() {
    let mut raw = vec![0; 256 * 1024];
    raw[MBR_TABLE + 4] = 0xee;
    raw[SECTOR..SECTOR + 8].copy_from_slice(GPT_SIGNATURE);
    raw[SECTOR + 72..SECTOR + 80].copy_from_slice(&2u64.to_le_bytes());
    raw[SECTOR + 80..SECTOR + 84].copy_from_slice(&4u32.to_le_bytes());
    raw[SECTOR + 84..SECTOR + 88].copy_from_slice(&128u32.to_le_bytes());
    let entry = 2 * SECTOR;
    raw[entry] = 1;
    raw[entry + 32..entry + 40].copy_from_slice(&40u64.to_le_bytes());
    raw[entry + 40..entry + 48].copy_from_slice(&99u64.to_le_bytes());
    let disk = snapshot_from_raw(&raw);

    let partitions = read_partitions(&disk).unwrap();

    assert_eq!(partitions[0].number, 1);
    assert_eq!(partitions[0].start_byte().unwrap(), 40 * SECTOR as u64);
    assert_eq!(partitions[0].len_bytes().unwrap(), 60 * SECTOR as u64);
}
