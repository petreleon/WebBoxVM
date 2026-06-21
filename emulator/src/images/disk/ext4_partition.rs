use super::partitions::Partition;
use crate::devices::virtio_blk::sparse_snapshot::SparseDiskSnapshot;
use ext4_view::Ext4Read;
use std::error::Error;
use std::fmt::{self, Display, Formatter};

#[derive(Clone)]
pub struct PartitionReader {
    disk: SparseDiskSnapshot,
    start: u64,
    len: u64,
}

impl PartitionReader {
    pub fn new(disk: SparseDiskSnapshot, partition: Partition) -> Result<Self, String> {
        Ok(Self {
            disk,
            start: partition.start_byte()?,
            len: partition.len_bytes()?,
        })
    }
}

impl Ext4Read for PartitionReader {
    fn read(
        &mut self,
        start_byte: u64,
        dst: &mut [u8],
    ) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
        if start_byte
            .checked_add(dst.len() as u64)
            .is_none_or(|end| end > self.len)
        {
            return Err(Box::new(PartitionReadError("read outside partition")));
        }
        self.disk
            .read_at(self.start + start_byte, dst)
            .map_err(|err| Box::new(PartitionReadString(err)) as _)
    }
}

#[derive(Debug)]
struct PartitionReadError(&'static str);

impl Display for PartitionReadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

impl Error for PartitionReadError {}

#[derive(Debug)]
struct PartitionReadString(String);

impl Display for PartitionReadString {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Error for PartitionReadString {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partition_reader_rejects_out_of_bounds_reads() {
        let partition = Partition {
            number: 1,
            start_lba: 0,
            end_lba: 0,
        };
        let snapshot = {
            let mut data = Vec::new();
            data.extend_from_slice(b"WBDISK01");
            data.extend_from_slice(&512u64.to_le_bytes());
            data.extend_from_slice(&(64u32 * 1024).to_le_bytes());
            data.extend_from_slice(&0u64.to_le_bytes());
            data
        };
        let disk = SparseDiskSnapshot::load(snapshot).unwrap();
        let mut reader = PartitionReader::new(disk, partition).unwrap();
        let mut out = [0; 1];

        assert!(reader.read(512, &mut out).is_err());
    }
}
