use super::*;

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
