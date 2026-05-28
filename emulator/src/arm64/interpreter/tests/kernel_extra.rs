use crate::arm64::{Armv8Cpu, decode, execute};
use crate::arm64::helpers;
use crate::arm64::opcodes::Opcode;
use crate::bus::SystemBus;

// ============================================================================
// Intensive instruction tests for EFI stub loop instructions
// ============================================================================

#[test]
fn test_sub_flags_cmp_w2_21() {
    // Reproduce: CMP W2, #21 where W2=5 should set C=0 (borrow occurred)
    let mut cpu = Armv8Cpu::new();
    let mut bus = SystemBus::new();

    // Set W2 = 5
    cpu.regs.set_w(2, 5);
    // CMP W2, #21  => SUBS WZR, W2, #21
    // raw = 0x7100545f
    let raw = 0x7100545f;
    let instr = decode(raw).unwrap();
    assert_eq!(instr.op, Opcode::CmpImm);
    assert_eq!(instr.rn, 2);
    assert_eq!(instr.imm, 21);
    assert!(!instr.sf);

    execute(&mut cpu, &mut bus, instr).unwrap();

    let c = cpu.pstate.c();
    assert!(!c, "CMP W2, #21 with W2=5: C should be 0 (borrow), got C={}", c);

    // Now BCond HS should NOT be taken
    let cond = 0b0010; // HS/CS
    assert!(!helpers::cond_taken(&cpu, cond),
        "BCond HS should NOT be taken when C=0");
}

#[test]
fn test_sub_flags_cmp_w2_157() {
    // CMP W2, #21 where W2=157 should set C=1 (no borrow)
    let mut cpu = Armv8Cpu::new();
    let mut bus = SystemBus::new();

    cpu.regs.set_w(2, 157);
    let raw = 0x7100545f;
    let instr = decode(raw).unwrap();
    execute(&mut cpu, &mut bus, instr).unwrap();

    let c = cpu.pstate.c();
    assert!(c, "CMP W2, #21 with W2=157: C should be 1 (no borrow), got C={}", c);

    let cond = 0b0010; // HS/CS
    assert!(helpers::cond_taken(&cpu, cond),
        "BCond HS SHOULD be taken when C=1");
}

#[test]
fn test_and_imm_w2_mask_ff() {
    // AND W2, W0, #0xFF where W0=0xFFFFFF9D => W2=0x9D=157
    let mut cpu = Armv8Cpu::new();
    let mut bus = SystemBus::new();

    cpu.regs.set_w(0, 0xFFFFFF9D_u32);
    let raw = 0x12001c02; // AND W2, W0, #0xFF
    let instr = decode(raw).unwrap();
    assert_eq!(instr.op, Opcode::AndImm);
    assert_eq!(instr.rd, 2);
    assert_eq!(instr.rn, 0);
    assert!(!instr.sf);

    execute(&mut cpu, &mut bus, instr).unwrap();
    assert_eq!(cpu.regs.w(2), 0x9D, "AND W2, W0, #0xFF with W0=0xFFFFFF9D");
    // X0 should be unchanged
    assert_eq!(cpu.regs.w(0), 0xFFFFFF9D_u32, "X0 should be unchanged by AND");
}

#[test]
fn test_sub_imm_w0_99() {
    // SUB W0, W0, #99 where W0=0 => W0=0xFFFFFF9D
    let mut cpu = Armv8Cpu::new();
    let mut bus = SystemBus::new();

    cpu.regs.set_w(0, 0);
    let raw = 0x51018c00; // SUB W0, W0, #99 (sf=0)
    let instr = decode(raw).unwrap();
    assert_eq!(instr.op, Opcode::SubImm);
    assert_eq!(instr.rd, 0);
    assert_eq!(instr.rn, 0);
    assert_eq!(instr.imm, 99);
    assert!(!instr.sf);

    execute(&mut cpu, &mut bus, instr).unwrap();
    assert_eq!(cpu.regs.w(0), 0xFFFFFF9D_u32, "SUB W0, W0, #99 with W0=0");
}

#[test]
fn test_adrp_instruction() {
    // ADRP X2, #offset at PC=0x41e27f1c
    // raw = 0x90000dc2
    // immhi = 0x1B8, immlo = 0
    // imm = 0x1B8 << 12 = 0x1B8000
    // page = 0x41e27000
    // result = 0x41e27000 + 0x1B8000 = 0x41FFF000? Let me check actual decode.
    let mut cpu = Armv8Cpu::new();
    let mut bus = SystemBus::new();

    cpu.regs.pc = 0x41e27f1c;
    let raw = 0x90000dc2;
    let instr = decode(raw).unwrap();
    assert_eq!(instr.op, Opcode::Adrp);
    assert_eq!(instr.rd, 2);

    execute(&mut cpu, &mut bus, instr).unwrap();
    let expected = (cpu.regs.pc & !0xFFF) + instr.imm;
    assert_eq!(cpu.regs.x(2), expected, "ADRP result mismatch");
}

#[test]
fn test_ldrb_x26_zero() {
    // LDRB W1, [X0] where X0=0 reads from address 0
    let mut cpu = Armv8Cpu::new();
    let mut bus = SystemBus::new();

    cpu.regs.set_x(0, 0);
    let raw = 0x39400001; // LDRB W1, [X0]
    let instr = decode(raw).unwrap();
    assert_eq!(instr.op, Opcode::Ldr);
    assert_eq!(instr.size, 1);
    assert_eq!(instr.rd, 1);
    assert_eq!(instr.rn, 0);

    execute(&mut cpu, &mut bus, instr).unwrap();
    // Address 0 in low region returns 0
    assert_eq!(cpu.regs.w(1), 0, "LDRB from address 0 should return 0");
}

#[test]
fn test_decode_eb21c01f() {
    let raw: u32 = 0xeb21c01f;
    let instr = decode(raw).unwrap();
    println!("raw=0x{:08x} op={:?} rd={} rn={} rm={}", raw, instr.op, instr.rd, instr.rn, instr.rm);
    assert_eq!(instr.op, Opcode::Cmp, "Expected Cmp, got {:?}", instr.op);
}

#[test]
fn test_decode_eb21c01f_debug() {
    let raw: u32 = 0xeb21c01f;
    let sf = ((raw >> 31) & 1) != 0;
    let op = (raw >> 30) & 1;
    let s = ((raw >> 29) & 1) != 0;
    let n = ((raw >> 21) & 1) != 0;
    let rd = (raw & 0x1F) as u8;
    println!("sf={} op={} s={} n={} rd={}", sf, op, s, n, rd);
    if s && op == 1 && rd == 31 {
        println!("Would be Cmp");
    } else {
        println!("Would NOT be Cmp");
    }
}

#[test]
fn test_decode_121d7820() {
    let raw: u32 = 0x121d7820;
    match decode(raw) {
        Some(instr) => println!("raw=0x{:08x} op={:?} rd={} rn={} imm={:#x}", raw, instr.op, instr.rd, instr.rn, instr.imm),
        None => println!("raw=0x{:08x} = None", raw),
    }
}

#[test]
fn test_fdt_header_verification_decoding() {
    use crate::loader::kernel::{load_kernel, KERNEL_LOAD};
    let mut bus = SystemBus::new();
    load_kernel(&mut bus, "/Users/petreleon/code/WebBoxVM/Image.gz").unwrap();

    let addrs = [0x41e29d90u64, 0x41e29d94u64, 0x41e29d98u64];
    for &addr in &addrs {
        let raw = bus.mem.read(addr, 4).unwrap() as u32;
        let instr = decode(raw);
        println!("ADDR={:#x} RAW={:#010x} DECODED={:?}", addr, raw, instr);
    }

    println!("\n--- Relocated loop diagnostics at 0x400b6e50 ---");
    for offset in (0..0x90).step_by(4) {
        let addr = 0x400b6e50u64 + offset;
        let raw = bus.mem.read(addr, 4).unwrap() as u32;
        let instr = decode(raw);
        println!("ADDR={:#x} RAW={:#010x} DECODED={:?}", addr, raw, instr);
    }
}

