use super::bootargs::{
    DISK_VIRTIO_MMIO_ARG, ISO_VIRTIO_MMIO_ARG, NET_VIRTIO_MMIO_ARG, ensure_serial_bootargs,
};
use super::images::prepare_kernel_image;
use super::installer_overlay::build_installer_network_overlay;
use super::*;
use crate::constants::ARM64_KERNEL_MAGIC;
use crate::initrd::parse_cpio;

fn arm64_image_stub() -> Vec<u8> {
    let mut image = vec![0u8; 64];
    image[56..60].copy_from_slice(&ARM64_KERNEL_MAGIC.to_le_bytes());
    image
}

#[test]
fn rejects_non_arm64_kernel() {
    let err = prepare_kernel_image(b"not a kernel").unwrap_err();
    assert!(err.contains("ARM64"));
}

#[test]
fn appends_serial_bootargs_when_missing() {
    let args = ensure_serial_bootargs("root=/dev/ram0 quiet");
    assert!(args.contains("root=/dev/ram0"));
    assert!(args.contains("earlycon=pl011,0x09000000"));
    assert!(args.contains("console=ttyAMA0,115200n8"));
    assert!(args.contains("loglevel=7"));
    assert!(args.contains("kvm-arm.mode=none"));
    assert!(args.contains("kvm.enable_virt_at_load=0"));
    assert!(args.contains("initcall_blacklist=finalize_pkvm,bpf_tcp_ca_kfunc_init"));
    assert!(args.contains("cryptomgr.notests=1"));
    assert!(args.contains(ISO_VIRTIO_MMIO_ARG));
    assert!(args.contains(DISK_VIRTIO_MMIO_ARG));
    assert!(args.contains(NET_VIRTIO_MMIO_ARG));
    assert!(args.contains("clocksource.arm_arch_timer.evtstrm=false"));
    assert!(args.contains("auto=false"));
    assert!(args.contains("---"));
    assert!(args.contains("DEBIAN_FRONTEND=text"));
    assert!(args.contains("TERM=vt102"));
}

#[test]
fn inserts_kernel_bootargs_before_debian_separator() {
    let args = ensure_serial_bootargs("--- quiet");
    let tokens: Vec<&str> = args.split_whitespace().collect();
    let separator = tokens.iter().position(|token| *token == "---").unwrap();

    assert!(tokens[..separator].contains(&"kvm-arm.mode=none"));
    assert!(tokens[..separator].contains(&"kvm.enable_virt_at_load=0"));
    assert!(
        tokens[..separator].contains(&"initcall_blacklist=finalize_pkvm,bpf_tcp_ca_kfunc_init")
    );
    assert!(tokens[..separator].contains(&"cryptomgr.notests=1"));
    assert!(tokens[..separator].contains(&ISO_VIRTIO_MMIO_ARG));
    assert!(tokens[..separator].contains(&DISK_VIRTIO_MMIO_ARG));
    assert!(tokens[..separator].contains(&NET_VIRTIO_MMIO_ARG));
    assert!(tokens[..separator].contains(&"clocksource.arm_arch_timer.evtstrm=false"));
    assert!(tokens[..separator].contains(&"auto=false"));
    assert_eq!(
        &tokens[separator..],
        &[
            "---",
            "TERM=vt102",
            "DEBIAN_FRONTEND=text",
            "console=ttyAMA0,115200n8",
        ]
    );
}

#[test]
fn extracts_kernel_and_initrd_from_minimal_iso() {
    let kernel = arm64_image_stub();
    let iso = filesystem::tests::minimal_iso_with_files(&[
        ("/vmlinuz", kernel.as_slice()),
        ("/initrd", b"initrd bytes"),
    ]);

    let boot = load_iso_boot_image(&iso).unwrap();
    assert_eq!(boot.kernel.len(), 64);
    assert!(boot.initrd.starts_with(b"initrd bytes"));
    assert_eq!(boot.kernel_path, "/vmlinuz");
    assert_eq!(boot.initrd_paths, vec!["/initrd"]);
    assert!(boot.bootargs.contains("console=ttyAMA0"));
}

#[test]
fn appends_depmod_wrapper_to_iso_initrd() {
    let kernel = arm64_image_stub();
    let original_initrd = b"initrd bytes";
    let iso = filesystem::tests::minimal_iso_with_files(&[
        ("/vmlinuz", kernel.as_slice()),
        ("/initrd", original_initrd.as_slice()),
    ]);

    let boot = load_iso_boot_image(&iso).unwrap();
    let overlay = &boot.initrd[original_initrd.len()..];
    let entries = parse_cpio(overlay).unwrap();
    let depmod = entries
        .iter()
        .find(|entry| entry.name == "sbin/depmod")
        .expect("depmod wrapper");

    assert_eq!(depmod.mode & 0o777, 0o755);
    let script = String::from_utf8(depmod.data.clone()).unwrap();
    assert!(script.contains("kernel/net/core/failover.ko*"));
    assert!(script.contains("kernel/drivers/net/net_failover.ko*"));
    assert!(script.contains("kernel/drivers/net/virtio_net.ko*"));
    assert!(!script.contains("depmod -a"));
}

#[test]
fn installer_network_overlay_is_a_single_executable_depmod() {
    let entries = parse_cpio(&build_installer_network_overlay()).unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].name, "sbin/depmod");
    assert_eq!(entries[0].mode & 0o777, 0o755);
}
