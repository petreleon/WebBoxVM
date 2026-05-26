use super::*;
use crate::arm64::{Armv8Cpu, decode, execute};
use crate::bus::SystemBus;

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

#[test]
#[ignore = "slow: loads 37 MB kernel"]
fn real_kernel_runs_past_prologue() {
    use crate::loader::kernel::{load_kernel, KERNEL_LOAD};
    use crate::efi::setup_efi_tables;

    let mut cpu = Armv8Cpu::new();
    let mut bus = SystemBus::new();

    let _entry = load_kernel(&mut bus, "/Users/petreleon/code/WebBoxVM/Image.gz").unwrap();

    let (handle, st) = setup_efi_tables(&mut bus, KERNEL_LOAD, 0x024f_0000);
    cpu.regs.set_x(0, handle);
    cpu.regs.set_x(1, st);
    cpu.regs.sp = 0x43FF_F000;

    bus.write(0x43FFE000, 4, 0xD65F03C0);
    cpu.regs.set_x(30, 0x43FFE000);

    cpu.regs.pc = KERNEL_LOAD + 0x01da7ee0;

    let mut steps = 0;
    for _ in 0..400 {
        let raw = match bus.read(cpu.regs.pc, 4) {
            Some(v) => v as u32,
            None => {
                println!("Memory fault at step {} PC=0x{:016x}", steps, cpu.regs.pc);
                break;
            }
        };
        if let Some(instr) = decode(raw) {
            if let Err(e) = execute(&mut cpu, &mut bus, instr) {
                println!("EXECUTE ERROR at step {} PC=0x{:016x}: {:?}", steps, cpu.regs.pc, e);
                break;
            }
            steps += 1;
            if steps >= 200 { break; }
        } else {
            println!("UNKNOWN INSTRUCTION at step {} PC=0x{:016x} raw=0x{:08x}", steps, cpu.regs.pc, raw);
            break;
        }
    }

    println!("EFI stub executed {} instructions, X0=0x{:016x}", steps, cpu.regs.x(0));
    assert!(steps >= 200, "Only executed {} instructions, expected at least 200", steps);
}

#[test]
fn synthetic_kernel_boots_to_uart() {
    use crate::loader::kernel::{load_raw_image, KERNEL_LOAD};
    use std::fs;

    let mut cpu = Armv8Cpu::new();
    let mut bus = SystemBus::new();

    let data = fs::read("/tmp/kernel_raw.bin")
        .expect("kernel not found; run build_kernel.sh first");
    load_raw_image(&mut bus, &data);

    cpu.regs.sp = 0x43FF_F000;

    let result = run(&mut cpu, &mut bus, KERNEL_LOAD, 50);
    println!("Result: {:?}", result);
    println!("UART output bytes: {:?}", bus.uart.output);
    assert!(result.is_ok(), "Synthetic kernel crashed: {:?}", result);
    assert!(bus.uart.output_string().contains("Uncompressing Linux..."), "UART output missing expected message");
}
