//! Instruction execution helpers: condition evaluation and register access.

use super::{Armv8Cpu, ProcessorState};

/// Evaluate an AArch64 condition code using current NZCV flags.
pub fn cond_taken(cpu: &Armv8Cpu, cond: u8) -> bool {
    let n = cpu.pstate.n();
    let z = cpu.pstate.z();
    let c = cpu.pstate.c();
    let v = cpu.pstate.v();
    match cond & 0xF {
        0b0000 => z,                    // EQ
        0b0001 => !z,                   // NE
        0b0010 => c,                    // CS/HS
        0b0011 => !c,                   // CC/LO
        0b0100 => n,                    // MI
        0b0101 => !n,                   // PL
        0b0110 => v,                    // VS
        0b0111 => !v,                   // VC
        0b1000 => c && !z,             // HI
        0b1001 => !c || z,              // LS
        0b1010 => n == v,               // GE
        0b1011 => n != v,               // LT
        0b1100 => !z && (n == v),      // GT
        0b1101 => z || (n != v),        // LE
        0b1110 => true,                // AL
        0b1111 => true,                // NV
        _ => true,
    }
}

/// Read general-purpose register (XZR = 0 when n >= 31).
pub fn read_reg(cpu: &Armv8Cpu, n: u8, sf: bool) -> u64 {
    let val = if n >= 31 { 0 } else { cpu.regs.x(n) };
    if sf { val } else { (val as u32) as u64 }
}

/// Read base register (SP when n==31, otherwise X[n]).
pub fn read_base(cpu: &Armv8Cpu, n: u8, sf: bool) -> u64 {
    let val = if n >= 31 { cpu.regs.sp } else { cpu.regs.x(n) };
    if sf { val } else { (val as u32) as u64 }
}

/// Write general-purpose register (SP when n==31).
pub fn write_reg(cpu: &mut Armv8Cpu, n: u8, val: u64, sf: bool) {
    if n >= 31 { cpu.regs.sp = val; }
    else if sf { cpu.regs.set_x(n, val); }
    else { cpu.regs.set_w(n, val as u32); }
}
