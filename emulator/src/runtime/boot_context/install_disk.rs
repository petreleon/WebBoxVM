use super::BootContext;
use crate::boot::merge_bootargs;
use crate::images::disk::installed_boot_from_snapshot;

const INSTALLED_DISK_COMPAT_BOOTARGS: &str = concat!(
    "lsm=landlock,lockdown,yama,integrity,apparmor ",
    "systemd.mask=keyboard-setup.service ",
    "systemd.mask=console-setup.service ",
    "systemd.mask=apparmor.service"
);

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
        installed.bootargs = installed_disk_bootargs(&installed.bootargs, extra_bootargs);
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

fn installed_disk_bootargs(base: &str, extra: &str) -> String {
    merge_bootargs(&merge_bootargs(base, INSTALLED_DISK_COMPAT_BOOTARGS), extra)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn installed_disk_bootargs_include_browser_compat_args() {
        let args = installed_disk_bootargs("root=/dev/vdb3", "");

        assert!(args.contains("lsm=landlock,lockdown,yama,integrity,apparmor"));
        assert!(args.contains("systemd.mask=keyboard-setup.service"));
        assert!(args.contains("systemd.mask=console-setup.service"));
        assert!(args.contains("systemd.mask=apparmor.service"));
    }

    #[test]
    fn installed_disk_bootargs_append_probe_args_last() {
        let args = installed_disk_bootargs("root=/dev/vdb3", "init=/bin/sh");

        assert!(args.ends_with("init=/bin/sh"));
    }
}
