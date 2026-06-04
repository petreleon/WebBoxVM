use super::*;

#[test]
fn run_add_sequence() {
    let mut cpu = Armv8Cpu::new();
    let mut bus = SystemBus::new();

    let code: [u32; 3] = [
        0xD280_0140, // MOVZ X0, #10
        0xD280_0401, // MOVZ X1, #32
        0x9A01_0002, // ADD X2, X0, X1
    ];

    for (i, &word) in code.iter().enumerate() {
        bus.mem.write(0x4000_0000 + (i as u64 * 4), 4, word as u64);
    }

    let steps = run(&mut cpu, &mut bus, 0x4000_0000, 3).unwrap();
    assert_eq!(steps, 3);
    assert_eq!(cpu.regs.x(2), 42);
}

#[test]
fn hello_uart() {
    let mut cpu = Armv8Cpu::new();
    let mut bus = SystemBus::new();

    let code: [u32; 4] = [
        0xD282_4680, // MOVZ X0, #0x1234
        0xD2A1_2001, // MOVZ X1, #0x0900, LSL #16
        0xF900_0020, // STR X0, [X1]
        0xD503_201F, // NOP
    ];

    for (i, &word) in code.iter().enumerate() {
        bus.mem.write(0x4000_0000 + (i as u64 * 4), 4, word as u64);
    }

    let steps = run(&mut cpu, &mut bus, 0x4000_0000, 4).unwrap();
    assert_eq!(steps, 4);
    assert_eq!(&bus.uart.output, &[0x34]);
    assert_eq!(bus.uart.output_string(), "4");
}

#[test]
fn boot_stub_to_kernel() {
    let mut cpu = Armv8Cpu::new();
    let mut bus = SystemBus::new();

    let boot_pc = 0x4000_0000u64;
    let kernel_pc = 0x4000_0100u64;
    let boot_stub = [0xD61F_0000u32]; // BR X0

    let kernel: [u32; 4] = [
        0xD282_4680, // MOVZ X0, #0x1234
        0xD2A1_2001, // MOVZ X1, #0x0900, LSL #16
        0xF900_0020, // STR X0, [X1]
        0xD503_201F, // NOP
    ];

    bus.mem.write(boot_pc, 4, boot_stub[0] as u64);
    for (i, &word) in kernel.iter().enumerate() {
        bus.mem.write(kernel_pc + (i as u64 * 4), 4, word as u64);
    }

    cpu.regs.set_x(0, kernel_pc);

    let steps = run(&mut cpu, &mut bus, boot_pc, 1).unwrap();
    assert_eq!(steps, 1);
    assert_eq!(cpu.regs.pc, kernel_pc);

    let _ = run(&mut cpu, &mut bus, kernel_pc, 3).unwrap();
    assert_eq!(bus.uart.output_string(), "4");
}
