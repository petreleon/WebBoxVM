use super::*;
use crate::constants::*;

fn synthetic_arm64_image(flags: u64) -> Vec<u8> {
    let mut image = vec![0u8; 64];
    let image_size = image.len() as u64;
    image[0..4].copy_from_slice(&0x1400_0000u32.to_le_bytes());
    image[8..16].copy_from_slice(&0u64.to_le_bytes());
    image[16..24].copy_from_slice(&image_size.to_le_bytes());
    image[24..32].copy_from_slice(&flags.to_le_bytes());
    image[56..60].copy_from_slice(&ARM64_KERNEL_MAGIC.to_le_bytes());
    image
}

#[test]
fn boot_plan_builds_artifacts_without_live_machine() {
    let image = synthetic_arm64_image(0);
    let initrd = [0x41u8, 0x42, 0x43];
    let plan =
        BootPlan::new_with_initrd_and_bootargs(&image, 2, &initrd, "console=ttyAMA0").unwrap();

    assert_eq!(plan.num_cores, 2);
    assert_eq!(plan.kernel_image, image);
    assert_eq!(plan.initrd_image, initrd);
    assert_eq!(plan.entry, KERNEL_LOAD_ADDR);
    assert_eq!(plan.dtb_addr, DTB_BASE);
    assert_eq!(plan.initrd_addr, INITRD_BASE);
    assert_eq!(plan.initrd_end, INITRD_BASE + initrd.len() as u64);
    assert!(plan.dtb_image.starts_with(&FDT_MAGIC.to_be_bytes()));
    assert!(plan.boot_media.is_none());
}

#[test]
fn boot_plan_dtb_uses_requested_core_count() {
    let image = synthetic_arm64_image(0);
    let initrd = [0x51u8; 4];
    let bootargs = "console=ttyAMA0";
    let plan = BootPlan::new_with_initrd_and_bootargs(&image, 4, &initrd, bootargs).unwrap();
    let expected = crate::dtb::build_dtb_with_boot_media_device_and_num_cores(
        RAM_BASE,
        RAM_SIZE,
        Some(INITRD_BASE),
        Some(INITRD_BASE + initrd.len() as u64),
        Some(bootargs),
        true,
        4,
    );

    assert_eq!(plan.dtb_image, expected);
    let text = String::from_utf8_lossy(&plan.dtb_image);
    assert!(text.contains("cpu@0"));
    assert!(text.contains("cpu@3"));
    assert!(!text.contains("cpu@4"));
}

#[test]
fn boot_plan_rejects_invalid_inputs_before_machine_creation() {
    let image = synthetic_arm64_image(0);

    assert!(BootPlan::new_with_initrd(&image, 0, &[1]).is_err());
    let too_many = BootPlan::new_with_initrd(&image, GICR_MAX_CPUS + 1, &[1]).unwrap_err();
    assert!(too_many.contains(&GICR_MAX_CPUS.to_string()));
    assert!(BootPlan::new_with_initrd(&image, 1, &[]).is_err());
}

#[test]
fn installed_disk_boot_plan_omits_empty_boot_media_device() {
    let image = synthetic_arm64_image(0);
    let initrd = [0x30u8; 4];
    let plan = BootPlan::new_installed_disk(&image, 1, &initrd, "root=UUID=test").unwrap();
    let text = String::from_utf8_lossy(&plan.dtb_image);

    assert!(!text.contains("virtio_blk@a000000"));
    assert!(text.contains("virtio_blk@a001000"));
    assert!(text.contains("virtio_net@a002000"));
}

#[test]
fn standard_boot_enters_non_relocatable_image_with_mmu_off() {
    let image = synthetic_arm64_image(0);
    let initrd = [0x30u8; 4];
    let mut ctx = BootContext::new_with_initrd(&image, 1, &initrd).unwrap();
    let cpu = &ctx.machine.cpus[0];

    assert_eq!(cpu.regs.pc, KERNEL_LOAD_ADDR);
    assert_eq!(cpu.regs.x(0), DTB_BASE);
    assert_eq!(cpu.regs.x(1), 0);
    assert_eq!(cpu.regs.x(2), 0);
    assert_eq!(cpu.regs.x(3), 0);
    assert_eq!(cpu.sys.sctlr_el1 & SCTLR_MMU_ENABLE, 0);
    assert_eq!(
        ctx.machine.bus.mem.read(KERNEL_LOAD_ADDR, 4),
        Some(0x1400_0000)
    );
    assert_eq!(ctx.run_efi_phase(100), 0);
}

#[test]
fn feeding_uart_input_queues_rx_and_injects_irq() {
    let mut ctx = BootContext::new(&[0u8; 64], 1).unwrap();

    ctx.feed_uart_input("ls\r");

    assert!(ctx.machine.bus.gic.is_pending(PL011_UART_IRQ_ID));
    assert_eq!(
        ctx.machine
            .bus
            .read(UART_BASE + UART_RIS_OFFSET, 4)
            .unwrap() as u16
            & (1 << 4),
        1 << 4
    );
    assert_eq!(
        ctx.machine.bus.read(UART_BASE + UART_DR_OFFSET, 4).unwrap() as u8,
        b'l'
    );
}
