use super::BootContext;
use crate::images::disk::installed_boot_from_snapshot;

impl BootContext {
    pub fn new_from_install_disk_snapshot(
        snapshot: Vec<u8>,
        num_cores: usize,
    ) -> Result<Self, String> {
        let installed = installed_boot_from_snapshot(snapshot)?;
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
