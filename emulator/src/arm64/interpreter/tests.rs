use super::*;
use crate::arm64::{Armv8Cpu, decode, execute, Opcode};
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
    let mut last_pc = cpu.regs.pc;
    for _ in 0..1000 {
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
            if cpu.regs.pc == last_pc {
                println!("Stalled at PC=0x{:016x} after {} steps", cpu.regs.pc, steps);
                break;
            }
            last_pc = cpu.regs.pc;
        } else {
            println!("UNKNOWN INSTRUCTION at step {} PC=0x{:016x} raw=0x{:08x}", steps, cpu.regs.pc, raw);
            break;
        }
    }

    println!("EFI stub executed {} instructions, X0=0x{:016x}", steps, cpu.regs.x(0));
    println!("  Final: PC=0x{:016x} SP=0x{:016x}", cpu.regs.pc, cpu.regs.sp);
    assert!(steps >= 200, "Only executed {} instructions, expected at least 200", steps);
}

#[test]
#[ignore = "slow: loads 37 MB kernel"]
fn real_kernel_runs_past_prologue_trace() {
    // Debug-only test: prints trace
    use crate::loader::kernel::{load_kernel, KERNEL_LOAD};
    use crate::efi::setup_efi_tables;

    let mut cpu = Armv8Cpu::new();
    let mut bus = SystemBus::new();

    let _entry = load_kernel(&mut bus, "/Users/petreleon/code/WebBoxVM/Image.gz").unwrap();

    let (handle, st) = setup_efi_tables(&mut bus, KERNEL_LOAD, 0x024f_0000);
    cpu.regs.set_x(0, handle);
    cpu.regs.set_x(1, st);
    cpu.regs.sp = 0x43FF_F000;
    // Seed X18 — EFI stub may dereference it via [X18, #offset].
    cpu.regs.set_x(18, st);

    bus.write(0x43FFE000, 4, 0xD65F03C0);
    cpu.regs.set_x(30, 0x43FFE000);

    cpu.regs.pc = KERNEL_LOAD + 0x01da7ee0;

    let mut steps = 0;
    let mut last_pc = cpu.regs.pc;
    for _ in 0..20000 {
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
            if cpu.regs.pc == last_pc {
                println!("Stalled at PC=0x{:016x} after {} steps", cpu.regs.pc, steps);
                break;
            }
            last_pc = cpu.regs.pc;
        } else {
            println!("UNKNOWN INSTRUCTION at step {} PC=0x{:016x} raw=0x{:08x}", steps, cpu.regs.pc, raw);
            break;
        }
    }

    println!("EFI stub executed {} instructions", steps);
    println!("  X0=0x{:016x} X1=0x{:016x} X2=0x{:016x} X18=0x{:016x}", cpu.regs.x(0), cpu.regs.x(1), cpu.regs.x(2), cpu.regs.x(18));
    println!("  PC=0x{:016x} SP=0x{:016x} X30=0x{:016x}", cpu.regs.pc, cpu.regs.sp, cpu.regs.x(30));
    println!("  UART output: {:?}", bus.uart.output_string());
    // No assert — this is just a debug trace
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

#[test]
fn test_mrs_msr_roundtrip() {
    let mut cpu = Armv8Cpu::new();
    let mut bus = SystemBus::new();

    // 1. Test MRS CurrentEL (read-only, EL3 by default)
    // CurrentEL: raw = 0xd5384241 (MRS X1, CurrentEL)
    // 0xd5384241: top12=0xD53, Rd=1 (X1), sysreg=0x4212
    let instr = decode(0xd5384241).unwrap();
    assert_eq!(instr.op, Opcode::Mrs);
    assert_eq!(instr.rd, 1);
    assert_eq!(instr.imm, 0x4212);
    execute(&mut cpu, &mut bus, instr).unwrap();
    assert_eq!(cpu.regs.x(1), 12); // EL3 << 2 = 12

    // 2. Test MSR SP_EL0 (write SP_EL0 with value from X2)
    // MSR SP_EL0, X2: raw = 0xd5184102
    // 0xd5184102: top12=0xD51, Rd=2 (X2), sysreg=0x4208
    cpu.regs.set_x(2, 0xCAFE_BABE_0000);
    let instr_msr = decode(0xd5184102).unwrap();
    assert_eq!(instr_msr.op, Opcode::Msr);
    assert_eq!(instr_msr.rd, 2);
    assert_eq!(instr_msr.imm, 0x4208);
    execute(&mut cpu, &mut bus, instr_msr).unwrap();
    assert_eq!(cpu.sys.sp_el0, 0xCAFE_BABE_0000);

    // 3. Test MRS X3, SP_EL0 (read SP_EL0 back into X3)
    // MRS X3, SP_EL0: raw = 0xd5384103
    // 0xd5384103: top12=0xD53, Rd=3 (X3), sysreg=0x4208
    let instr_mrs = decode(0xd5384103).unwrap();
    assert_eq!(instr_mrs.op, Opcode::Mrs);
    assert_eq!(instr_mrs.rd, 3);
    assert_eq!(instr_mrs.imm, 0x4208);
    execute(&mut cpu, &mut bus, instr_mrs).unwrap();
    assert_eq!(cpu.regs.x(3), 0xCAFE_BABE_0000);
}

#[test]
fn test_madd_msub() {
    let mut cpu = Armv8Cpu::new();
    let mut bus = SystemBus::new();

    // 1. Test MADD W0, W1, W2, W3 (32-bit: W0 = W3 + W1 * W2)
    // raw = 0x1B020C20 -> MADD W0, W1, W2, W3 (rd=0, rn=1, rm=2, ra=3, sf=false, size=0)
    cpu.regs.set_x(1, 10);
    cpu.regs.set_x(2, 3);
    cpu.regs.set_x(3, 5);
    let instr_madd = decode(0x1B02_0C20).unwrap();
    assert_eq!(instr_madd.op, Opcode::Madd);
    assert_eq!(instr_madd.rd, 0);
    assert_eq!(instr_madd.rn, 1);
    assert_eq!(instr_madd.rm, 2);
    assert_eq!(instr_madd.cond, 3); // Ra
    assert_eq!(instr_madd.sf, false);
    assert_eq!(instr_madd.size, 0);
    execute(&mut cpu, &mut bus, instr_madd).unwrap();
    assert_eq!(cpu.regs.x(0), 35); // 5 + 10 * 3 = 35

    // 2. Test MSUB X0, X1, X2, X3 (64-bit: X0 = X3 - X1 * X2)
    // raw = 0x9B028C20 -> MSUB X0, X1, X2, X3 (rd=0, rn=1, rm=2, ra=3, sf=true, size=0)
    cpu.regs.set_x(1, 4);
    cpu.regs.set_x(2, 5);
    cpu.regs.set_x(3, 30);
    let instr_msub = decode(0x9B02_8C20).unwrap();
    assert_eq!(instr_msub.op, Opcode::Msub);
    assert_eq!(instr_msub.sf, true);
    assert_eq!(instr_msub.size, 0);
    execute(&mut cpu, &mut bus, instr_msub).unwrap();
    assert_eq!(cpu.regs.x(0), 10); // 30 - 4 * 5 = 10

    // 3. Test UMADDL X21, W21, W24, XZR (UMULL X21, W21, W24)
    // raw = 0x9bb87eb5 (rd=21, rn=21, rm=24, ra=31, sf=true, size=1)
    cpu.regs.set_x(21, 0xFFFFFFFF_00000003); // W21 is 3
    cpu.regs.set_x(24, 0x4);                 // W24 is 4
    let instr_umull = decode(0x9bb87eb5).unwrap();
    assert_eq!(instr_umull.op, Opcode::Madd);
    assert_eq!(instr_umull.rd, 21);
    assert_eq!(instr_umull.rn, 21);
    assert_eq!(instr_umull.rm, 24);
    assert_eq!(instr_umull.cond, 31); // XZR
    assert_eq!(instr_umull.sf, true);
    assert_eq!(instr_umull.size, 1);  // UMADDL
    execute(&mut cpu, &mut bus, instr_umull).unwrap();
    assert_eq!(cpu.regs.x(21), 12);   // 0 + 3 * 4 = 12
}
