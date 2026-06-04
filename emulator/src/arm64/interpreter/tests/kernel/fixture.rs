use super::*;

pub(super) struct KernelFixture {
    pub(super) cpu: Armv8Cpu,
    pub(super) bus: SystemBus,
    pub(super) dtb_addr: u64,
}

pub(super) fn load_real_kernel_fixture(
    init_script: Vec<u8>,
    set_el1: bool,
    set_x18_to_system_table: bool,
) -> KernelFixture {
    use crate::dtb::{build_dtb, load_dtb};
    use crate::efi::setup_efi_tables;
    use crate::initrd::{build_cpio, load_initrd};
    use crate::loader::kernel::{KERNEL_LOAD, load_kernel};

    let mut cpu = Armv8Cpu::new();
    if set_el1 {
        cpu.pstate = cpu.pstate.with_el(1);
    }
    let mut bus = SystemBus::new();

    let entry = load_kernel(
        &mut bus,
        concat!(env!("CARGO_MANIFEST_DIR"), "/../.artifacts/Image"),
    )
    .unwrap();

    let dtb_addr = 0x4700_0000u64;
    let (handle, st) = setup_efi_tables(&mut bus, KERNEL_LOAD, 0x024f_0000, dtb_addr);
    cpu.regs.set_x(0, handle);
    cpu.regs.set_x(1, st);
    cpu.regs.sp = 0x43F0_0000;
    if set_x18_to_system_table {
        cpu.regs.set_x(18, st);
    }

    bus.write(0x43EFE000, 4, 0xD65F03C0);
    cpu.regs.set_x(30, 0x43EFE000);
    setup_boot_page_tables(&mut cpu, &mut bus);
    cpu.regs.pc = entry;

    let busybox_data = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../.artifacts/busybox-aarch64"
    ))
    .unwrap_or_else(|_| vec![0u8; 100]);
    let entries = vec![
        ("bin/busybox".to_string(), busybox_data.clone(), 0o100755u32),
        ("bin/sh".to_string(), busybox_data, 0o100755u32),
        ("init".to_string(), init_script, 0o100755u32),
        ("proc".to_string(), Vec::new(), 0o040755u32),
        ("sys".to_string(), Vec::new(), 0o040755u32),
    ];
    let cpio = build_cpio(&entries);
    let initrd_start = 0x4400_0000u64;
    let initrd_end = initrd_start + cpio.len() as u64;
    let dtb = build_dtb(
        0x4000_0000,
        0x4000_0000,
        Some(initrd_start),
        Some(initrd_end),
        Some("earlycon=pl011,0x09000000 console=ttyAMA0 rdinit=/init"),
    );

    load_initrd(&mut bus, initrd_start, &cpio);
    load_dtb(&mut bus, dtb_addr, &dtb);

    KernelFixture { cpu, bus, dtb_addr }
}
