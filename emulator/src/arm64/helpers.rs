//! Instruction execution helpers — condition evaluation and register access.
//!
//! These are used by execute.rs and encapsulate the most common ARM64 patterns:
//!   - Is a condition true given the current NZCV flags?
//!   - Read/write a register (handling XZR/WZR zero-register semantics)

use crate::constants::*;
use super::Armv8Cpu;

/// Evaluate an AArch64 condition code against the current NZCV flags.
///
/// ARM64 has 16 condition codes, encoded in the `cond` field (4 bits) of
/// branch, select, and compare instructions.
///
/// | Code  | Mnemonic | Meaning                            | Formula     |
/// |-------|----------|------------------------------------|-------------|
/// | 0000  | EQ       | Equal (Z set)                      | Z           |
/// | 0001  | NE       | Not Equal (Z clear)                | !Z          |
/// | 0010  | CS / HS  | Carry Set / Unsigned Higher or Same| C           |
/// | 0011  | CC / LO  | Carry Clear / Unsigned Lower       | !C          |
/// | 0100  | MI       | Minus (N set)                      | N           |
/// | 0101  | PL       | Plus (N clear)                     | !N          |
/// | 0110  | VS       | Overflow Set                       | V           |
/// | 0111  | VC       | Overflow Clear                     | !V          |
/// | 1000  | HI       | Unsigned Higher                    | C && !Z     |
/// | 1001  | LS       | Unsigned Lower or Same             | !C || Z     |
/// | 1010  | GE       | Signed Greater or Equal            | N == V      |
/// | 1011  | LT       | Signed Less Than                   | N != V      |
/// | 1100  | GT       | Signed Greater Than                | !Z && N==V  |
/// | 1101  | LE       | Signed Less or Equal               | Z || N!=V   |
/// | 1110  | AL       | Always                             | true        |
/// | 1111  | NV       | Always (alias)                     | true        |
pub fn cond_taken(cpu: &Armv8Cpu, cond: u8) -> bool {
    let n = cpu.pstate.n();
    let z = cpu.pstate.z();
    let c = cpu.pstate.c();
    let v = cpu.pstate.v();
    match cond & 0xF {
        0b0000 => z,                    // EQ
        0b0001 => !z,                   // NE
        0b0010 => c,                    // CS / HS
        0b0011 => !c,                   // CC / LO
        0b0100 => n,                    // MI
        0b0101 => !n,                   // PL
        0b0110 => v,                    // VS
        0b0111 => !v,                   // VC
        0b1000 => c && !z,              // HI
        0b1001 => !c || z,              // LS
        0b1010 => n == v,               // GE
        0b1011 => n != v,               // LT
        0b1100 => !z && (n == v),       // GT
        0b1101 => z || (n != v),        // LE
        0b1110 => true,                 // AL
        0b1111 => true,                 // NV
        _ => true,
    }
}

/// Read a general-purpose register.  Returns 0 (the zero register) when `n >= 31`.
///
/// If `sf` (sixty-four bit flag) is false, the value is zero-extended from 32 bits.
pub fn read_reg(cpu: &Armv8Cpu, n: u8, sf: bool) -> u64 {
    let val = if n >= ZERO_REGISTER_INDEX { 0 } else { cpu.regs.x(n) };
    if sf { val } else { (val as u32) as u64 }
}

/// Read a base register for address computation.  Returns SP (stack pointer)
/// when `n == 31`, otherwise returns X[n].
///
/// This is the behavior used by LDR/STR/LDP/STP and ADD/SUB (extended register).
pub fn read_base(cpu: &Armv8Cpu, n: u8, sf: bool) -> u64 {
    let val = if n >= SP_REGISTER_INDEX { cpu.regs.sp } else { cpu.regs.x(n) };
    if sf { val } else { (val as u32) as u64 }
}

/// Write a general-purpose register or SP (if n == 31).  Used by address-forming
/// instructions (ADD/SUB extended, LDR/STR writeback, etc.).
pub fn write_reg_sp(cpu: &mut Armv8Cpu, n: u8, val: u64, sf: bool) {
    if n >= SP_REGISTER_INDEX {
        cpu.regs.sp = if sf { val } else { (val as u32) as u64 };
    } else if sf {
        cpu.regs.set_x(n, val);
    } else {
        cpu.regs.set_w(n, val as u32);
    }
}

/// Write a general-purpose register.  Writing to register 31 (XZR/WZR) is a no-op.
pub fn write_reg(cpu: &mut Armv8Cpu, n: u8, val: u64, sf: bool) {
    if n >= ZERO_REGISTER_INDEX {
        // Writing to XZR / WZR discards the value — ARM64 idiom.
    } else if sf {
        cpu.regs.set_x(n, val);
    } else {
        cpu.regs.set_w(n, val as u32);
    }
}
