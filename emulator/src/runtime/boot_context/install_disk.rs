use super::BootContext;
use crate::boot::{BootPlan, merge_bootargs};
use crate::images::disk::{InstalledDiskBoot, installed_boot_from_snapshot};

const INSTALLED_DISK_COMPAT_BOOTARGS: &str = concat!(
    "lsm=landlock,lockdown,yama,integrity,apparmor ",
    "systemd.mask=keyboard-setup.service ",
    "systemd.mask=console-setup.service ",
    "systemd.mask=apparmor.service ",
    "systemd.mask=getty-static.service ",
    "systemd.mask=getty@tty1.service ",
    "systemd.mask=getty@tty2.service ",
    "systemd.mask=getty@tty3.service ",
    "systemd.mask=getty@tty4.service ",
    "systemd.mask=getty@tty5.service ",
    "systemd.mask=getty@tty6.service"
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
        let installed = installed_boot_from_snapshot(snapshot)?;
        Self::from_installed_disk_boot(installed, num_cores, extra_bootargs)
    }

    fn from_installed_disk_boot(
        mut installed: InstalledDiskBoot,
        num_cores: usize,
        extra_bootargs: &str,
    ) -> Result<Self, String> {
        installed.bootargs = installed_disk_bootargs(&installed.bootargs, extra_bootargs);
        let mut ctx = Self::from_plan(BootPlan::new_installed_disk(
            &installed.kernel,
            num_cores,
            &installed.initrd,
            &installed.bootargs,
        )?)?;
        ctx.machine
            .bus
            .virtio_disk
            .set_sparse_disk_snapshot(installed.disk);
        Ok(ctx)
    }
}

fn installed_disk_bootargs(base: &str, extra: &str) -> String {
    merge_bootargs(&merge_bootargs(base, INSTALLED_DISK_COMPAT_BOOTARGS), extra)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::devices::virtio_blk::sparse_snapshot::SparseDiskSnapshot;

    #[test]
    fn installed_disk_bootargs_include_browser_compat_args() {
        let args = installed_disk_bootargs("root=/dev/vdb3", "");

        assert!(args.contains("lsm=landlock,lockdown,yama,integrity,apparmor"));
        assert!(args.contains("systemd.mask=keyboard-setup.service"));
        assert!(args.contains("systemd.mask=console-setup.service"));
        assert!(args.contains("systemd.mask=apparmor.service"));
        assert!(args.contains("systemd.mask=getty-static.service"));
        assert!(args.contains("systemd.mask=getty@tty6.service"));
    }

    #[test]
    fn installed_disk_bootargs_append_probe_args_last() {
        let args = installed_disk_bootargs("root=/dev/vdb3", "init=/bin/sh");

        assert!(args.ends_with("init=/bin/sh"));
    }

    #[test]
    fn installed_boot_retains_parsed_snapshot_as_disk_base() {
        let disk = SparseDiskSnapshot::load(empty_snapshot(64 * 1024)).unwrap();
        let backing = disk.clone();
        let installed = InstalledDiskBoot {
            disk,
            kernel: vec![0; 64],
            initrd: vec![1],
            bootargs: "root=/dev/vda1".into(),
            boot_partition: 1,
            root_partition: Some(1),
        };

        let ctx = BootContext::from_installed_disk_boot(installed, 1, "").unwrap();

        assert!(
            ctx.machine
                .bus
                .virtio_disk
                .sparse_disk_shares_snapshot_backing(&backing)
        );
        assert_eq!(ctx.install_disk_generation(), 1);
    }

    fn empty_snapshot(size_bytes: u64) -> Vec<u8> {
        let mut snapshot = Vec::new();
        snapshot.extend_from_slice(b"WBDISK01");
        snapshot.extend_from_slice(&size_bytes.to_le_bytes());
        snapshot.extend_from_slice(&(64 * 1024u32).to_le_bytes());
        snapshot.extend_from_slice(&0u64.to_le_bytes());
        snapshot
    }
}
