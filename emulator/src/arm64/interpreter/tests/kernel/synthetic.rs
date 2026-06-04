use super::*;

#[test]
fn synthetic_kernel_boots_to_uart() {
    use crate::loader::kernel::{KERNEL_LOAD, load_raw_image};

    let mut cpu = Armv8Cpu::new();
    let mut bus = SystemBus::new();

    let message = b"Uncompressing Linux...\n";
    let msg_offset = (2 + message.len() * 2 + 1) * 4;
    let mut words = vec![0xD2A8_0001, 0xD2A1_2002];
    for i in 0..message.len() {
        words.push(0x3940_0020 | (((msg_offset + i) as u32) << 10));
        words.push(0x3800_0040);
    }
    words.push(0x1400_0000);

    let mut data: Vec<u8> = words.iter().flat_map(|&word| word.to_le_bytes()).collect();
    data.extend_from_slice(message);
    load_raw_image(&mut bus, &data);

    cpu.regs.sp = 0x43F0_0000;

    let result = run(&mut cpu, &mut bus, KERNEL_LOAD, words.len() + 2);
    println!("Result: {:?}", result);
    println!("UART output bytes: {:?}", bus.uart.output);
    assert!(result.is_ok(), "Synthetic kernel crashed: {:?}", result);
    assert!(
        bus.uart.output_string().contains("Uncompressing Linux..."),
        "UART output missing expected message"
    );
}

#[test]
fn synthetic_kernel_reads_initrd_from_dtb() {
    use crate::dtb::{build_dtb, load_dtb};
    use crate::initrd::{build_cpio, load_initrd};
    use crate::loader::kernel::{KERNEL_LOAD, load_raw_image};

    let mut cpu = Armv8Cpu::new();
    let mut bus = SystemBus::new();

    let entries = vec![("init".to_string(), b"hello from initrd".to_vec(), 0o755u32)];
    let cpio = build_cpio(&entries);
    let initrd_start = 0x4200_0000u64;
    let initrd_end = initrd_start + cpio.len() as u64;

    let dtb = build_dtb(
        0x4000_0000,
        0x4000_0000,
        Some(initrd_start),
        Some(initrd_end),
        Some("earlycon console=ttyAMA0"),
    );
    let dtb_addr = 0x4800_0000u64;

    load_initrd(&mut bus, initrd_start, &cpio);
    load_dtb(&mut bus, dtb_addr, &dtb);

    let kernel: Vec<u32> = vec![
        0xD2A12002, 0xD2A84003, 0x39400060, 0x38000040, 0x39400460, 0x38000040, 0x14000000,
    ];
    let kernel_bytes: Vec<u8> = kernel.iter().flat_map(|&w| w.to_le_bytes()).collect();
    load_raw_image(&mut bus, &kernel_bytes);

    cpu.regs.set_x(0, dtb_addr);
    cpu.regs.sp = 0x43F0_0000;

    let result = run(&mut cpu, &mut bus, KERNEL_LOAD, 20);
    println!("Result: {:?}", result);
    println!("UART output bytes: {:?}", bus.uart.output);
    println!("UART output string: {:?}", bus.uart.output_string());

    assert!(result.is_ok(), "Synthetic kernel crashed: {:?}", result);
    assert_eq!(
        bus.uart.output,
        vec![b'0', b'7'],
        "Expected first two bytes of cpio magic on UART"
    );
}
