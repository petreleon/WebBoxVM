use super::*;
use crate::devices::virtio_blk::sparse_snapshot::SparseDiskSnapshot;
use crate::initrd::{CpioNode, build_cpio_nodes};

#[test]
fn installed_disk_bootargs_include_browser_compat_args() {
    let args = installed_disk_bootargs("root=/dev/vdb3", "", false);

    assert!(args.contains("lsm=landlock,lockdown,yama,integrity,apparmor"));
    assert!(!args.contains("maxcpus="));
    assert!(args.contains("systemd.mask=keyboard-setup.service"));
    assert!(args.contains("systemd.mask=console-setup.service"));
    assert!(args.contains("systemd.mask=apparmor.service"));
    assert!(args.contains("systemd.mask=getty-static.service"));
    assert!(args.contains("systemd.mask=getty@tty6.service"));
}

#[test]
fn probe_args_and_staged_smp_have_unambiguous_precedence() {
    let probed = installed_disk_bootargs("root=/dev/vdb3", "init=/bin/sh", false);
    let staged = installed_disk_bootargs("root=/dev/vdb3", "debug", true);

    assert!(probed.ends_with("init=/bin/sh"));
    assert!(!probed.contains("maxcpus="));
    assert!(staged.contains("maxcpus=1"));
    assert!(staged.contains("noresume"));
    assert!(staged.ends_with("clocksource.arm_arch_timer.evtstrm=false"));
}

#[test]
fn staging_requires_exact_generated_bootargs_without_overrides() {
    let base = generated_bootargs();
    for extra in [
        "debug",
        "maxcpus=2",
        "init=/bin/sh",
        "nr_cpus=1",
        "nosmp",
        "rdinit=/bin/sh",
        "systemd.mask=serial-getty@ttyAMA0.service",
    ] {
        assert!(!staged_smp_bootargs_allowed(base, extra));
    }
    assert!(staged_smp_bootargs_allowed(base, ""));
    assert!(!staged_smp_bootargs_allowed("root=/dev/vda3", ""));
    assert!(!staged_smp_bootargs_allowed(
        "root=/dev/vda3 rw rootwait TERM=vt102 console=ttyAMA0,115200n8 nosmp",
        ""
    ));
}

#[test]
fn explicit_staging_requires_compatible_two_core_boot() {
    let installed = installed_boot(compatible_initrd());
    for (cores, extra, requested, expected) in [
        (2, "", true, true),
        (2, "", false, false),
        (2, "maxcpus=2", true, false),
        (3, "", true, false),
    ] {
        let (_ctx, staged) =
            BootContext::from_installed_disk_boot(installed.clone(), cores, extra, requested)
                .unwrap();
        assert_eq!(staged, expected);
    }

    let mut rootless = installed;
    rootless.root_partition = None;
    let (_ctx, staged) = BootContext::from_installed_disk_boot(rootless, 2, "", true).unwrap();
    assert!(!staged);
}

#[test]
fn installed_boot_retains_parsed_snapshot_as_disk_base() {
    let installed = installed_boot(vec![1]);
    let backing = installed.disk.clone();
    let (ctx, staged) = BootContext::from_installed_disk_boot(installed, 1, "", false).unwrap();

    assert!(!staged);
    assert!(
        ctx.machine
            .bus
            .virtio_disk
            .sparse_disk_shares_snapshot_backing(&backing)
    );
    assert_eq!(ctx.install_disk_generation(), 1);
}

#[test]
fn unavailable_fast_initrd_preserves_the_current_overlay_fallback() {
    let mut installed = installed_boot(compatible_initrd());
    let original = installed.initrd.clone();
    let overlay_start = (original.len() + 3) & !3;

    prepare_staged_initrd(&mut installed);

    assert_eq!(&installed.initrd[..original.len()], original);
    let overlay = crate::initrd::parse_cpio(&installed.initrd[overlay_start..]).unwrap();
    assert_eq!(overlay[0].name, "conf/param.conf");
}

fn compatible_initrd() -> Vec<u8> {
    build_cpio_nodes(&[
        CpioNode::file(
            "init",
            b"run_scripts /scripts/init-bottom\nmount -n -o move /run ${rootmnt}/run",
            0o755,
        ),
        CpioNode::file(
            "scripts/init-bottom/ORDER",
            b"[ -e /conf/param.conf ] && . /conf/param.conf",
            0o644,
        ),
    ])
}

fn installed_boot(initrd: Vec<u8>) -> InstalledDiskBoot {
    InstalledDiskBoot {
        disk: SparseDiskSnapshot::load(empty_snapshot(64 * 1024)).unwrap(),
        kernel: vec![0; 64],
        initrd,
        bootargs: generated_bootargs().into(),
        boot_partition: 1,
        root_partition: Some(1),
        staged_smp_capable: true,
        kernel_suffix: None,
        fast_initrd_kernel: false,
        root_ext4_clean: false,
    }
}

fn generated_bootargs() -> &'static str {
    "root=/dev/vda1 rw rootwait TERM=vt102 console=ttyAMA0,115200n8"
}

fn empty_snapshot(size_bytes: u64) -> Vec<u8> {
    let mut snapshot = Vec::new();
    snapshot.extend_from_slice(b"WBDISK01");
    snapshot.extend_from_slice(&size_bytes.to_le_bytes());
    snapshot.extend_from_slice(&(64 * 1024u32).to_le_bytes());
    snapshot.extend_from_slice(&0u64.to_le_bytes());
    snapshot
}
