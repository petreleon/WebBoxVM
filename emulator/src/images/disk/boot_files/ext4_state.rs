use super::{Partition, SparseDiskSnapshot};

const SUPERBLOCK_OFFSET: u64 = 1024;
const SUPERBLOCK_BYTES: usize = 104;
const MAGIC_OFFSET: usize = 56;
const STATE_OFFSET: usize = 58;
const INCOMPAT_OFFSET: usize = 96;
const EXT4_MAGIC: u16 = 0xef53;
const EXT4_VALID_FS: u16 = 0x0001;
#[cfg(test)]
const EXT4_ERROR_FS: u16 = 0x0002;
#[cfg(test)]
const EXT4_ORPHAN_FS: u16 = 0x0004;
const EXT4_FEATURE_INCOMPAT_RECOVER: u32 = 0x0004;

pub(super) fn clean(disk: &SparseDiskSnapshot, partition: Partition) -> bool {
    let Some(offset) = partition
        .start_byte()
        .ok()
        .and_then(|start| start.checked_add(SUPERBLOCK_OFFSET))
    else {
        return false;
    };
    let mut superblock = [0; SUPERBLOCK_BYTES];
    disk.read_at(offset, &mut superblock).is_ok() && clean_superblock(&superblock)
}

fn clean_superblock(superblock: &[u8]) -> bool {
    le_u16(superblock, MAGIC_OFFSET) == Some(EXT4_MAGIC)
        && le_u16(superblock, STATE_OFFSET) == Some(EXT4_VALID_FS)
        && le_u32(superblock, INCOMPAT_OFFSET)
            .is_some_and(|features| features & EXT4_FEATURE_INCOMPAT_RECOVER == 0)
}

fn le_u16(bytes: &[u8], offset: usize) -> Option<u16> {
    Some(u16::from_le_bytes(
        bytes.get(offset..offset + 2)?.try_into().ok()?,
    ))
}

fn le_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    Some(u32::from_le_bytes(
        bytes.get(offset..offset + 4)?.try_into().ok()?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_gate_requires_valid_state_without_recovery() {
        let mut superblock = [0; SUPERBLOCK_BYTES];
        superblock[MAGIC_OFFSET..MAGIC_OFFSET + 2].copy_from_slice(&EXT4_MAGIC.to_le_bytes());
        superblock[STATE_OFFSET..STATE_OFFSET + 2].copy_from_slice(&EXT4_VALID_FS.to_le_bytes());
        assert!(clean_superblock(&superblock));

        superblock[STATE_OFFSET..STATE_OFFSET + 2].copy_from_slice(&0u16.to_le_bytes());
        assert!(!clean_superblock(&superblock));
        superblock[STATE_OFFSET..STATE_OFFSET + 2]
            .copy_from_slice(&(EXT4_VALID_FS | EXT4_ERROR_FS).to_le_bytes());
        assert!(!clean_superblock(&superblock));
        superblock[STATE_OFFSET..STATE_OFFSET + 2]
            .copy_from_slice(&(EXT4_VALID_FS | EXT4_ORPHAN_FS).to_le_bytes());
        assert!(!clean_superblock(&superblock));
        superblock[STATE_OFFSET..STATE_OFFSET + 2].copy_from_slice(&EXT4_VALID_FS.to_le_bytes());
        superblock[INCOMPAT_OFFSET..INCOMPAT_OFFSET + 4]
            .copy_from_slice(&EXT4_FEATURE_INCOMPAT_RECOVER.to_le_bytes());
        assert!(!clean_superblock(&superblock));
    }
}
