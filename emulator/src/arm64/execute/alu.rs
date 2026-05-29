//! Data-processing helpers: logical, bitfield, multiply, shift, bit manipulation, flags.

use crate::arm64::helpers::{read_reg, write_reg, read_base, write_reg_sp, cond_taken};
use crate::arm64::Armv8Cpu;
use crate::constants::*;
use super::{Instr, Opcode};

#[derive(Copy, Clone)]
pub(super) enum ShiftDir { Left, Right, ArithRight, RotateRight }

// ── Flag-setting helpers ──

fn sign_bit(sf: bool) -> u32 {
    if sf { SIGN_BIT_64 } else { SIGN_BIT_32 }
}

pub(super) fn set_nz_flags(cpu: &mut Armv8Cpu, val: u64, sf: bool) {
    let sb = sign_bit(sf);
    let is_zero = if sf { val == 0 } else { (val as u32) == 0 };
    cpu.pstate.set_nzcv(((val >> sb) & 1) != 0, is_zero, false, false);
}

pub(super) fn add_flags(cpu: &mut Armv8Cpu, lhs: u64, rhs: u64, sf: bool) -> u64 {
    let val = lhs.wrapping_add(rhs);
    let sb = sign_bit(sf);
    let n = ((val >> sb) & 1) != 0;
    let z = if sf { val == 0 } else { (val as u32) == 0 };
    let c = if sf { val < lhs } else { (val as u32) < (lhs as u32) };
    let sign_mask = 1u64 << sb;
    let v = (lhs & sign_mask) == (rhs & sign_mask) && (lhs & sign_mask) != (val & sign_mask);
    cpu.pstate.set_nzcv(n, z, c, v);
    val
}

pub(super) fn sub_flags(cpu: &mut Armv8Cpu, lhs: u64, rhs: u64, sf: bool) -> u64 {
    let val = lhs.wrapping_sub(rhs);
    let sb = sign_bit(sf);
    let n = ((val >> sb) & 1) != 0;
    let z = if sf { val == 0 } else { (val as u32) == 0 };
    let c = if sf { lhs >= rhs } else { (lhs as u32) >= (rhs as u32) };
    let sign_mask = 1u64 << sb;
    let v = (lhs & sign_mask) != (rhs & sign_mask) && (lhs & sign_mask) != (val & sign_mask);
    cpu.pstate.set_nzcv(n, z, c, v);
    val
}

// ── Register extension & shifting ──

pub(super) fn extend_reg_val(cpu: &Armv8Cpu, rm: u8, option: u8, shift: u8, sf: bool) -> u64 {
    let mut val = read_reg(cpu, rm, if option == 3 || option == 7 { sf } else { option >= 2 });
    val = match option {
        0 => (val as u8) as u64,              // UXTB
        1 => (val as u16) as u64,             // UXTH
        2 => (val as u32) as u64,             // UXTW
        3 => val,                              // UXTX
        4 => ((val as i8) as i64) as u64,     // SXTB
        5 => ((val as i16) as i64) as u64,    // SXTH
        6 => ((val as i32) as i64) as u64,    // SXTW
        7 => val,                              // SXTX
        _ => val,
    };
    if sf { val << shift } else { ((val as u32) << shift) as u64 }
}

pub(super) fn shifted_reg_val(cpu: &Armv8Cpu, rm: u8, shift_type: u8, amount: u8, sf: bool) -> u64 {
    let val = read_reg(cpu, rm, sf);
    let amount = amount as u32;
    if amount == 0 { return val; }
    match shift_type {
        0 => if sf { val << amount } else { ((val as u32) << amount) as u64 },
        1 => if sf { val >> amount } else { ((val as u32) >> amount) as u64 },
        2 => if sf { ((val as i64) >> amount) as u64 } else { (((val as u32) as i32) >> amount) as u64 },
        3 => if sf { val.rotate_right(amount) } else { (val as u32).rotate_right(amount) as u64 },
        _ => val,
    }
}

pub(super) fn ext_or_shifted_val(cpu: &Armv8Cpu, rm: u8, cond: u8, amount: u8, sf: bool) -> u64 {
    if cond >= 4 {
        extend_reg_val(cpu, rm, cond, amount, sf)
    } else {
        shifted_reg_val(cpu, rm, cond, amount, sf)
    }
}

// ── Logical register ──

pub(super) fn exec_logical_reg(cpu: &mut Armv8Cpu, instr: Instr) {
    let n = (instr.cond & 4) != 0;
    let shift_type = instr.cond & 3;
    let mut rhs = shifted_reg_val(cpu, instr.rm, shift_type, instr.imm as u8, instr.sf);
    if n { rhs = !rhs; if !instr.sf { rhs &= 0xFFFFFFFF; } }
    let lhs = read_reg(cpu, instr.rn, instr.sf);
    let val = match instr.op {
        Opcode::AndReg => lhs & rhs,
        Opcode::OrrReg => lhs | rhs,
        Opcode::EorReg => lhs ^ rhs,
        Opcode::AndsReg => { set_nz_flags(cpu, lhs & rhs, instr.sf); lhs & rhs }
        _ => unreachable!(),
    };
    write_reg(cpu, instr.rd, val, instr.sf);
}

// ── Bitfield ──

pub(super) fn exec_bitfield(cpu: &mut Armv8Cpu, instr: Instr) {
    let size = if instr.sf { 64 } else { 32 };
    let r = instr.rm as u32;
    let s = instr.imm as u32;
    let src = read_reg(cpu, instr.rn, instr.sf);

    let val = match instr.op {
        Opcode::Ubfm => bitfield_extract(src, r, s, size, false),
        Opcode::Sbfm => bitfield_extract(src, r, s, size, true),
        Opcode::Bfm  => {
            let dst = read_reg(cpu, instr.rd, instr.sf);
            bitfield_insert(dst, src, r, s, size)
        }
        _ => unreachable!(),
    };
    write_reg(cpu, instr.rd, val, instr.sf);
}

fn bitfield_extract(src: u64, r: u32, s: u32, size: u32, signed: bool) -> u64 {
    let result = if s >= r {
        let len = s - r + 1;
        let mask = bitmask(len);
        let extracted = (src >> r) & mask;
        if signed { sign_extend(extracted, s - r, size) } else { extracted }
    } else {
        let len = s + 1;
        let mask = bitmask(len);
        let shift = size - r;
        let extracted = (src & mask) << shift;
        if signed { sign_extend(extracted, shift + s, size) } else { extracted }
    };
    word_truncate(result, size)
}

fn bitfield_insert(dst: u64, src: u64, r: u32, s: u32, size: u32) -> u64 {
    let result = if s >= r {
        let len = s - r + 1;
        let mask = bitmask(len);
        let dst_mask = !(mask << r);
        (dst & dst_mask) | ((src & mask) << r)
    } else {
        let len = s + 1;
        let mask = bitmask(len);
        let shift = size - r;
        let dst_mask = !(mask << shift);
        (dst & dst_mask) | ((src & mask) << shift)
    };
    word_truncate(result, size)
}

fn bitmask(len: u32) -> u64 {
    if len >= 64 { !0 } else { (1u64 << len) - 1 }
}

fn sign_extend(val: u64, sign_bit: u32, size: u32) -> u64 {
    if sign_bit < 63 && (val & (1u64 << sign_bit)) != 0 {
        let extend_mask = !((1u64 << (sign_bit + 1)) - 1);
        val | (extend_mask & full_width_mask(size))
    } else {
        val
    }
}

fn word_truncate(val: u64, size: u32) -> u64 {
    if size == 64 { val } else { val & WORD_MASK }
}

fn full_width_mask(size: u32) -> u64 {
    if size == 64 { !0 } else { WORD_MASK }
}

// ── Conditional compare ──

pub(super) fn exec_condcmp(cpu: &mut Armv8Cpu, instr: Instr) {
    if cond_taken(cpu, instr.cond) {
        let lhs = read_reg(cpu, instr.rn, instr.sf);
        let rhs = if instr.size == 1 {
            instr.rm as u64
        } else {
            read_reg(cpu, instr.rm, instr.sf)
        };
        if instr.op == Opcode::Ccmn {
            let _ = add_flags(cpu, lhs, rhs, instr.sf);
        } else {
            let _ = sub_flags(cpu, lhs, rhs, instr.sf);
        }
    } else {
        let n = (instr.imm & 8) != 0;
        let z = (instr.imm & 4) != 0;
        let c = (instr.imm & 2) != 0;
        let v = (instr.imm & 1) != 0;
        cpu.pstate.set_nzcv(n, z, c, v);
    }
}

// ── Multiply ──

pub(super) fn exec_madd(cpu: &mut Armv8Cpu, instr: Instr) {
    let sf_src = instr.size == 0 && instr.sf;
    let n = read_reg(cpu, instr.rn, sf_src);
    let m = read_reg(cpu, instr.rm, sf_src);
    let a = read_reg(cpu, instr.cond, instr.sf);
    let val = match instr.size {
        0 => if instr.sf { a.wrapping_add(n.wrapping_mul(m)) } else { ((a as u32).wrapping_add((n as u32).wrapping_mul(m as u32))) as u64 },
        1 => a.wrapping_add((n as u32 as u64).wrapping_mul(m as u32 as u64)),
        2 => a.wrapping_add(((n as u32 as i32) as i64).wrapping_mul((m as u32 as i32) as i64) as u64),
        _ => return,
    };
    write_reg(cpu, instr.rd, val, instr.sf);
}

pub(super) fn exec_msub(cpu: &mut Armv8Cpu, instr: Instr) {
    let sf_src = instr.size == 0 && instr.sf;
    let n = read_reg(cpu, instr.rn, sf_src);
    let m = read_reg(cpu, instr.rm, sf_src);
    let a = read_reg(cpu, instr.cond, instr.sf);
    let val = match instr.size {
        0 => if instr.sf { a.wrapping_sub(n.wrapping_mul(m)) } else { ((a as u32).wrapping_sub((n as u32).wrapping_mul(m as u32))) as u64 },
        1 => a.wrapping_sub((n as u32 as u64).wrapping_mul(m as u32 as u64)),
        2 => a.wrapping_sub(((n as u32 as i32) as i64).wrapping_mul((m as u32 as i32) as i64) as u64),
        _ => return,
    };
    write_reg(cpu, instr.rd, val, instr.sf);
}

// ── Variable shift ──

pub(super) fn exec_variable_shift(cpu: &mut Armv8Cpu, instr: Instr, dir: ShiftDir) {
    let n_val = read_reg(cpu, instr.rn, instr.sf);
    let m_val = read_reg(cpu, instr.rm, instr.sf);
    let val = if instr.sf {
        let shift = (m_val & 63) as u32;
        match dir {
            ShiftDir::Left => n_val << shift,
            ShiftDir::Right => n_val >> shift,
            ShiftDir::ArithRight => ((n_val as i64) >> shift) as u64,
            ShiftDir::RotateRight => n_val.rotate_right(shift),
        }
    } else {
        let shift = (m_val & 31) as u32;
        match dir {
            ShiftDir::Left => ((n_val as u32) << shift) as u64,
            ShiftDir::Right => ((n_val as u32) >> shift) as u64,
            ShiftDir::ArithRight => ((n_val as i32) >> shift) as u32 as u64,
            ShiftDir::RotateRight => (n_val as u32).rotate_right(shift) as u64,
        }
    };
    write_reg(cpu, instr.rd, val, instr.sf);
}

// ── Divide ──

pub(super) fn exec_div(cpu: &mut Armv8Cpu, instr: Instr, signed: bool) {
    let n = read_reg(cpu, instr.rn, instr.sf);
    let m = read_reg(cpu, instr.rm, instr.sf);
    let val = if m == 0 {
        0
    } else if instr.sf {
        if signed { (n as i64).checked_div(m as i64).unwrap_or(n as i64) as u64 } else { n / m }
    } else {
        if signed { (n as i32).checked_div(m as i32).unwrap_or(n as i32) as u32 as u64 } else { ((n as u32) / (m as u32)) as u64 }
    };
    write_reg(cpu, instr.rd, val, instr.sf);
}

// ── Reverse bits/bytes ──

pub(super) fn exec_rev(cpu: &mut Armv8Cpu, instr: Instr) {
    if instr.sf {
        write_reg(cpu, instr.rd, read_reg(cpu, instr.rn, true).swap_bytes(), true);
    } else {
        write_reg(cpu, instr.rd, (read_reg(cpu, instr.rn, false) as u32).swap_bytes() as u64, false);
    }
}

pub(super) fn exec_rev16(cpu: &mut Armv8Cpu, instr: Instr) {
    const MASK_EVEN: u64 = 0xFF00_FF00_FF00_FF00;
    const MASK_ODD:  u64 = 0x00FF_00FF_00FF_00FF;
    if instr.sf {
        let val = read_reg(cpu, instr.rn, true);
        write_reg(cpu, instr.rd, ((val & MASK_EVEN) >> 8) | ((val & MASK_ODD) << 8), true);
    } else {
        let val = read_reg(cpu, instr.rn, false) as u32;
        write_reg(cpu, instr.rd, (((val & 0xFF00_FF00) >> 8) | ((val & 0x00FF_00FF) << 8)) as u64, false);
    }
}

pub(super) fn exec_rbit(cpu: &mut Armv8Cpu, instr: Instr) {
    if instr.sf {
        write_reg(cpu, instr.rd, read_reg(cpu, instr.rn, true).reverse_bits(), true);
    } else {
        write_reg(cpu, instr.rd, (read_reg(cpu, instr.rn, false) as u32).reverse_bits() as u64, false);
    }
}

pub(super) fn exec_clz(cpu: &mut Armv8Cpu, instr: Instr) {
    if instr.sf {
        write_reg(cpu, instr.rd, read_reg(cpu, instr.rn, true).leading_zeros() as u64, true);
    } else {
        write_reg(cpu, instr.rd, (read_reg(cpu, instr.rn, false) as u32).leading_zeros() as u64, false);
    }
}
