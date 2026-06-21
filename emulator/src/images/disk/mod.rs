mod boot_files;
mod ext4_partition;
mod partitions;
#[cfg(test)]
mod tests;

use crate::devices::virtio_blk::sparse_snapshot::SparseDiskSnapshot;
pub use boot_files::InstalledDiskBoot;

pub fn installed_boot_from_snapshot(snapshot: Vec<u8>) -> Result<InstalledDiskBoot, String> {
    let disk = SparseDiskSnapshot::load(snapshot)?;
    let partitions = partitions::read_partitions(&disk)?;
    boot_files::extract_installed_boot(disk, &partitions)
}
