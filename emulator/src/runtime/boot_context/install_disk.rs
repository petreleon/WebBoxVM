use super::BootContext;
use super::fast_boot::{STAGED_SMP_BOOTARGS, append_staged_smp_overlay, staged_smp_supported};
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
        Self::new_from_install_disk_snapshot_with_staged_smp(
            snapshot,
            num_cores,
            extra_bootargs,
            false,
        )
        .map(|(context, _staged)| context)
    }

    /// Build an installed-disk boot and report whether guarded staged SMP was enabled.
    pub fn new_from_install_disk_snapshot_with_staged_smp(
        snapshot: Vec<u8>,
        num_cores: usize,
        extra_bootargs: &str,
        staged_smp_requested: bool,
    ) -> Result<(Self, bool), String> {
        let installed = installed_boot_from_snapshot(snapshot)?;
        Self::from_installed_disk_boot(installed, num_cores, extra_bootargs, staged_smp_requested)
    }

    fn from_installed_disk_boot(
        mut installed: InstalledDiskBoot,
        num_cores: usize,
        extra_bootargs: &str,
        staged_smp_requested: bool,
    ) -> Result<(Self, bool), String> {
        let staged = staged_smp_requested
            && installed.staged_smp_capable
            && installed.root_partition.is_some()
            && staged_smp_supported(&installed.initrd, num_cores)
            && staged_smp_bootargs_allowed(&installed.bootargs, extra_bootargs);
        installed.bootargs = installed_disk_bootargs(&installed.bootargs, extra_bootargs, staged);
        if staged {
            append_staged_smp_overlay(&mut installed.initrd);
        }
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
        Ok((ctx, staged))
    }
}

fn installed_disk_bootargs(base: &str, extra: &str, staged_smp: bool) -> String {
    let compat = merge_bootargs(base, INSTALLED_DISK_COMPAT_BOOTARGS);
    let combined = merge_bootargs(&compat, extra);
    if staged_smp {
        merge_bootargs(&combined, STAGED_SMP_BOOTARGS)
    } else {
        combined
    }
}

fn staged_smp_bootargs_allowed(base: &str, extra: &str) -> bool {
    if !extra.trim().is_empty() {
        return false;
    }
    let mut root = 0;
    let mut serial_console = 0;
    let mut rw = false;
    let mut rootwait = false;
    let mut term = false;
    for arg in base.split_ascii_whitespace() {
        match arg {
            "rw" => rw = true,
            "rootwait" => rootwait = true,
            "TERM=vt102" => term = true,
            "console=ttyAMA0,115200n8" => serial_console += 1,
            _ if arg
                .strip_prefix("root=")
                .is_some_and(|value| !value.is_empty()) =>
            {
                root += 1;
            }
            _ => return false,
        }
    }
    root == 1 && serial_console == 1 && rw && rootwait && term
}

#[cfg(test)]
mod tests;
