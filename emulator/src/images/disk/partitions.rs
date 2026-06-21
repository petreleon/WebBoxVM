use crate::devices::virtio_blk::sparse_snapshot::SparseDiskSnapshot;

const SECTOR_SIZE: u64 = 512;
const MBR_PARTITION_TABLE: usize = 446;
const MBR_ENTRY_LEN: usize = 16;
const GPT_HEADER_LBA: u64 = 1;
const GPT_SIGNATURE: &[u8; 8] = b"EFI PART";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Partition {
    pub number: u32,
    pub start_lba: u64,
    pub end_lba: u64,
}

impl Partition {
    pub fn start_byte(self) -> Result<u64, String> {
        self.start_lba
            .checked_mul(SECTOR_SIZE)
            .ok_or_else(|| "partition start overflows".to_string())
    }

    pub fn len_bytes(self) -> Result<u64, String> {
        self.end_lba
            .checked_sub(self.start_lba)
            .and_then(|last| last.checked_add(1))
            .and_then(|sectors| sectors.checked_mul(SECTOR_SIZE))
            .ok_or_else(|| "partition length overflows".to_string())
    }
}

pub fn read_partitions(disk: &SparseDiskSnapshot) -> Result<Vec<Partition>, String> {
    let mbr = read_sector(disk, 0)?;
    if has_protective_mbr(&mbr) {
        return read_gpt_partitions(disk);
    }
    read_mbr_partitions(&mbr)
}

fn has_protective_mbr(mbr: &[u8]) -> bool {
    (0..4).any(|slot| mbr[MBR_PARTITION_TABLE + slot * MBR_ENTRY_LEN + 4] == 0xee)
}

fn read_mbr_partitions(mbr: &[u8]) -> Result<Vec<Partition>, String> {
    let mut partitions = Vec::new();
    for slot in 0..4 {
        let offset = MBR_PARTITION_TABLE + slot * MBR_ENTRY_LEN;
        let part_type = mbr[offset + 4];
        let start = le_u32(mbr, offset + 8) as u64;
        let sectors = le_u32(mbr, offset + 12) as u64;
        if part_type == 0 || sectors == 0 {
            continue;
        }
        let end_lba = start
            .checked_add(sectors - 1)
            .ok_or_else(|| "MBR partition end overflows".to_string())?;
        partitions.push(Partition {
            number: slot as u32 + 1,
            start_lba: start,
            end_lba,
        });
    }
    Ok(partitions)
}

fn read_gpt_partitions(disk: &SparseDiskSnapshot) -> Result<Vec<Partition>, String> {
    let header = read_sector(disk, GPT_HEADER_LBA)?;
    if &header[..8] != GPT_SIGNATURE {
        return Err("GPT header signature missing".to_string());
    }
    let entries_lba = le_u64(&header, 72);
    let entry_count = le_u32(&header, 80).min(512);
    let entry_size = le_u32(&header, 84) as usize;
    if entry_size < 128 || entry_size > 4096 {
        return Err("GPT partition entry size is unsupported".to_string());
    }

    let mut partitions = Vec::new();
    for index in 0..entry_count {
        let offset = entries_lba
            .checked_mul(SECTOR_SIZE)
            .and_then(|base| base.checked_add(index as u64 * entry_size as u64))
            .ok_or_else(|| "GPT partition entry offset overflows".to_string())?;
        let mut entry = vec![0; entry_size];
        disk.read_at(offset, &mut entry)?;
        if entry[..16].iter().all(|byte| *byte == 0) {
            continue;
        }
        let start_lba = le_u64(&entry, 32);
        let end_lba = le_u64(&entry, 40);
        if start_lba <= end_lba {
            partitions.push(Partition {
                number: index + 1,
                start_lba,
                end_lba,
            });
        }
    }
    Ok(partitions)
}

fn read_sector(disk: &SparseDiskSnapshot, lba: u64) -> Result<[u8; 512], String> {
    let mut sector = [0; 512];
    disk.read_at(lba * SECTOR_SIZE, &mut sector)?;
    Ok(sector)
}

fn le_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

fn le_u64(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap())
}
