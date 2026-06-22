use super::BootContext;
use crate::boot::merge_bootargs;
use crate::images::disk::installed_boot_from_snapshot;

impl BootContext {
    pub fn new_from_install_disk_snapshot(
        snapshot: Vec<u8>,
        num_cores: usize,
    ) -> Result<Self, String> {
        Self::new_from_install_disk_snapshot_with_extra_bootargs(snapshot, num_cores, "")
    }

    pub fn new_from_install_disk_snapshot_with_extra_bootargs(
        snapshot: Vec<u8>,
        num_cores: usize,
        extra_bootargs: &str,
    ) -> Result<Self, String> {
        let mut installed = installed_boot_from_snapshot(snapshot)?;
        installed.bootargs = merge_bootargs(&installed.bootargs, extra_bootargs);
        let mut ctx = Self::new_with_initrd_and_bootargs(
            &installed.kernel,
            num_cores,
            &installed.initrd,
            &installed.bootargs,
        )?;
        ctx.restore_install_disk(installed.disk.as_bytes())?;
        Ok(ctx)
    }
}
