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
