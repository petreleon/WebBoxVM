//! Data-processing helpers: logical, bitfield, multiply, shift, bit manipulation, flags.

use super::{Instr, Opcode};
use crate::arm64::Armv8Cpu;
use crate::arm64::helpers::{cond_taken, read_reg, write_reg};
use crate::constants::*;

#[derive(Copy, Clone)]
pub(super) enum ShiftDir {
    Left,
    Right,
    ArithRight,
    RotateRight,
}

// ── Flag-setting helpers ──

fn sign_bit(sf: bool) -> u32 {
    if sf { SIGN_BIT_64 } else { SIGN_BIT_32 }
}

pub(super) fn set_nz_flags(cpu: &mut Armv8Cpu, val: u64, sf: bool) {
    let sb = sign_bit(sf);
    let is_zero = if sf { val == 0 } else { (val as u32) == 0 };
    cpu.pstate
        .set_nzcv(((val >> sb) & 1) != 0, is_zero, false, false);
}

pub(super) fn add_flags(cpu: &mut Armv8Cpu, lhs: u64, rhs: u64, sf: bool) -> u64 {
    let val = lhs.wrapping_add(rhs);
    let sb = sign_bit(sf);
    let n = ((val >> sb) & 1) != 0;
    let z = if sf { val == 0 } else { (val as u32) == 0 };
    let c = if sf {
        val < lhs
    } else {
        (val as u32) < (lhs as u32)
    };
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
    let c = if sf {
        lhs >= rhs
    } else {
        (lhs as u32) >= (rhs as u32)
    };
    let sign_mask = 1u64 << sb;
    let v = (lhs & sign_mask) != (rhs & sign_mask) && (lhs & sign_mask) != (val & sign_mask);
    cpu.pstate.set_nzcv(n, z, c, v);
    val
}

pub(super) fn exec_addsub_carry(cpu: &mut Armv8Cpu, instr: Instr) {
    let carry = u64::from(cpu.pstate.c());
    let mask = if instr.sf { u64::MAX } else { WORD_MASK };
    let lhs = read_reg(cpu, instr.rn, instr.sf) & mask;
    let rhs_raw = read_reg(cpu, instr.rm, instr.sf) & mask;
    let rhs = match instr.op {
        Opcode::Adc | Opcode::Adcs => rhs_raw,
        Opcode::Sbc | Opcode::Sbcs => !rhs_raw & mask,
        _ => unreachable!(),
    };
    let wide = lhs as u128 + rhs as u128 + carry as u128;
    let result = (wide & mask as u128) as u64;

    if matches!(instr.op, Opcode::Adcs | Opcode::Sbcs) {
        let sign_mask = 1u64 << sign_bit(instr.sf);
        let n = (result & sign_mask) != 0;
        let z = result == 0;
        let c = wide > mask as u128;
        let v = (lhs & sign_mask) == (rhs & sign_mask) && (lhs & sign_mask) != (result & sign_mask);
        cpu.pstate.set_nzcv(n, z, c, v);
    }

    write_reg(cpu, instr.rd, result, instr.sf);
}

pub(super) fn simd_replicate_byte(byte: u8) -> u128 {
    let mut value = 0u128;
    for lane in 0..16 {
        value |= (byte as u128) << (lane * 8);
    }
    value
}

pub(super) fn exec_simd_data(cpu: &mut Armv8Cpu, instr: Instr) {
    let rd = instr.rd as usize;
    let rn = instr.rn as usize;
    let rm = instr.rm as usize;

    match instr.op {
        Opcode::SimdDupByte => {
            let element_size = if instr.cond == 0 {
                1
            } else {
                instr.cond as usize
            };
            let value = read_reg(cpu, instr.rn, element_size == 8) as u128;
            cpu.simd[rd] = simd_replicate_element(value, element_size, instr.size as usize);
        }
        Opcode::SimdDupElem => {
            let element_size = instr.cond.max(1) as usize;
            let value = simd_element(cpu.simd[rn], instr.imm as usize, element_size);
            cpu.simd[rd] = simd_replicate_element(value, element_size, instr.size as usize);
        }
        Opcode::SimdFmovReg64 => {
            cpu.simd[rd] = read_fp_bits(cpu, instr.rn, instr.size) as u128;
        }
        Opcode::SimdFmovGprToD => {
            cpu.simd[rd] = read_reg(cpu, instr.rn, true) as u128;
        }
        Opcode::SimdFmovGprToS => {
            let value = read_reg(cpu, instr.rn, instr.size == 8) as u128;
            cpu.simd[rd] = value & simd_element_mask(instr.size as usize);
        }
        Opcode::SimdFmovDToGpr => {
            write_reg(cpu, instr.rd, cpu.simd[rn] as u64, true);
        }
        Opcode::SimdFmovSToGpr => {
            let value = (cpu.simd[rn] & simd_element_mask(instr.size as usize)) as u64;
            write_reg(cpu, instr.rd, value, instr.size == 8);
        }
        Opcode::SimdFmovLaneToGpr => {
            let shift = (instr.imm as u32) * 64;
            write_reg(cpu, instr.rd, (cpu.simd[rn] >> shift) as u64, true);
        }
        Opcode::SimdUmov => {
            let element_size = instr.cond.max(1) as u32;
            let shift = (instr.imm as u32) * element_size * 8;
            let bits = element_size * 8;
            let mask = if bits == 64 {
                u64::MAX as u128
            } else {
                (1u128 << bits) - 1
            };
            let value = ((cpu.simd[rn] >> shift) & mask) as u64;
            write_reg(cpu, instr.rd, value, instr.sf);
        }
        Opcode::SimdInsGprLane => {
            let shift = (instr.imm as u32) * 64;
            let mask = (u64::MAX as u128) << shift;
            let value = (read_reg(cpu, instr.rn, true) as u128) << shift;
            cpu.simd[rd] = (cpu.simd[rd] & !mask) | (value & mask);
        }
        Opcode::SimdCmeqZero => {
            let src = cpu.simd[rn];
            let element_size = instr.imm.max(1) as usize;
            cpu.simd[rd] = simd_compare_elements_with_zero(src, element_size, instr.size as usize);
        }
        Opcode::SimdCmeqReg => {
            let lhs = cpu.simd[rn];
            let rhs = cpu.simd[rm];
            let element_size = instr.imm.max(1) as usize;
            cpu.simd[rd] = simd_elementwise_binary(
                lhs,
                rhs,
                element_size,
                instr.size as usize,
                |a, b, mask| if a == b { mask } else { 0 },
            );
        }
        Opcode::SimdCmhsReg => {
            let lhs = cpu.simd[rn];
            let rhs = cpu.simd[rm];
            cpu.simd[rd] = simd_compare_vec_bytes(lhs, rhs, |a, b| a >= b);
        }
        Opcode::SimdUqsub => {
            const FPSR_QC: u64 = 1 << 27;

            let element_size = instr.imm.max(1) as usize;
            let mask = simd_element_mask(element_size);
            let lhs = cpu.simd[rn] & mask;
            let rhs = cpu.simd[rm] & mask;
            let (value, saturated) = if lhs < rhs {
                (0, true)
            } else {
                (lhs - rhs, false)
            };
            if saturated {
                cpu.sys.fpsr |= FPSR_QC;
            }
            cpu.simd[rd] = value;
        }
        Opcode::SimdFcvtzu => {
            let value = if instr.size == 4 {
                f32::from_bits(read_fp_bits(cpu, instr.rn, 4) as u32).trunc() as u32 as u64
            } else {
                f64::from_bits(read_fp_bits(cpu, instr.rn, 8)).trunc() as u64
            };
            write_fp_bits(cpu, instr.rd, value, instr.size);
        }
        Opcode::SimdShrn => {
            let src = cpu.simd[rn];
            let shift = instr.imm as u32;
            let mut out = 0u128;
            for lane in 0..8 {
                let half = ((src >> (lane * 16)) & 0xffff) as u16;
                out |= (((half >> shift) & 0xff) as u128) << (lane * 8);
            }
            cpu.simd[rd] = out;
        }
        Opcode::SimdAddhn => {
            let lhs = cpu.simd[rn];
            let rhs = cpu.simd[rm];
            let mut out = 0u128;
            for lane in 0..8 {
                let a = ((lhs >> (lane * 16)) & 0xffff) as u16;
                let b = ((rhs >> (lane * 16)) & 0xffff) as u16;
                out |= ((((a as u32 + b as u32) >> 8) & 0xff) as u128) << (lane * 8);
            }
            cpu.simd[rd] = out;
        }
        Opcode::SimdAddVec => {
            cpu.simd[rd] = simd_elementwise_binary(
                cpu.simd[rn],
                cpu.simd[rm],
                instr.imm.max(1) as usize,
                instr.size as usize,
                |a, b, mask| a.wrapping_add(b) & mask,
            );
        }
        Opcode::SimdSubVec => {
            cpu.simd[rd] = simd_elementwise_binary(
                cpu.simd[rn],
                cpu.simd[rm],
                instr.imm.max(1) as usize,
                instr.size as usize,
                |a, b, mask| a.wrapping_sub(b) & mask,
            );
        }
        Opcode::SimdAddp => {
            let lhs = cpu.simd[rn];
            let rhs = cpu.simd[rm];
            let element_size = instr.imm.max(1) as usize;
            cpu.simd[rd] =
                simd_pairwise_binary(lhs, rhs, element_size, instr.size as usize, |a, b, mask| {
                    a.wrapping_add(b) & mask
                });
        }
        Opcode::SimdAddv => {
            let element_size = instr.imm.max(1) as usize;
            let bits = element_size * 8;
            let element_mask = if bits == 128 {
                u128::MAX
            } else {
                (1u128 << bits) - 1
            };
            let lanes = instr.size as usize / element_size;
            let mut sum = 0u128;
            for lane in 0..lanes {
                sum =
                    sum.wrapping_add(simd_element(cpu.simd[rn], lane, element_size)) & element_mask;
            }
            cpu.simd[rd] = sum;
        }
        Opcode::SimdExt => {
            let lhs = cpu.simd[rn];
            let rhs = cpu.simd[rm];
            let vector_size = instr.size as usize;
            let index = (instr.imm as usize).min(vector_size.saturating_sub(1));
            let mut out = 0u128;
            for lane in 0..vector_size {
                let source_index = index + lane;
                let byte = if source_index < vector_size {
                    simd_byte(lhs, source_index)
                } else {
                    simd_byte(rhs, source_index - vector_size)
                };
                out |= (byte as u128) << (lane * 8);
            }
            cpu.simd[rd] = out;
        }
        Opcode::SimdUmaxp => {
            let lhs = cpu.simd[rn];
            let rhs = cpu.simd[rm];
            let element_size = instr.imm.max(1) as usize;
            cpu.simd[rd] =
                simd_pairwise_binary(lhs, rhs, element_size, instr.size as usize, |a, b, _| {
                    a.max(b)
                });
        }
        Opcode::SimdUminp => {
            let lhs = cpu.simd[rn];
            let rhs = cpu.simd[rm];
            let element_size = instr.imm.max(1) as usize;
            cpu.simd[rd] =
                simd_pairwise_binary(lhs, rhs, element_size, instr.size as usize, |a, b, _| {
                    a.min(b)
                });
        }
        Opcode::SimdCnt => {
            let vector_size = instr.size as usize;
            let mut out = 0u128;
            for lane in 0..vector_size {
                out |= (simd_byte(cpu.simd[rn], lane).count_ones() as u128) << (lane * 8);
            }
            cpu.simd[rd] = out;
        }
        Opcode::SimdCmtst => {
            cpu.simd[rd] = simd_elementwise_binary(
                cpu.simd[rn],
                cpu.simd[rm],
                instr.imm.max(1) as usize,
                instr.size as usize,
                |a, b, mask| if (a & b) != 0 { mask } else { 0 },
            );
        }
        Opcode::SimdShlImm => {
            let element_size = instr.cond.max(1) as usize;
            let bits = element_size * 8;
            let shift = instr.imm as usize;
            let lanes = (instr.size as usize / element_size).max(1);
            let element_mask = simd_element_mask(element_size);
            let mut out = 0u128;
            for lane in 0..lanes {
                let value = simd_element(cpu.simd[rn], lane, element_size);
                let shifted = if shift >= bits { 0 } else { value << shift };
                out |= (shifted & element_mask) << (lane * bits);
            }
            cpu.simd[rd] = out & simd_vector_mask(instr.size as usize);
        }
        Opcode::SimdSli => {
            let element_size = instr.cond.max(1) as usize;
            let bits = element_size * 8;
            let shift = instr.imm as usize;
            let lanes = (instr.size as usize / element_size).max(1);
            let element_mask = simd_element_mask(element_size);
            let preserve_mask = if shift >= bits {
                element_mask
            } else {
                (1u128 << shift) - 1
            };
            let mut out = cpu.simd[rd];
            for lane in 0..lanes {
                let source = simd_element(cpu.simd[rn], lane, element_size);
                let dest = simd_element(cpu.simd[rd], lane, element_size);
                let inserted = if shift >= bits {
                    0
                } else {
                    (source << shift) & element_mask
                };
                let value = inserted | (dest & preserve_mask);
                out &= !(element_mask << (lane * bits));
                out |= value << (lane * bits);
            }
            cpu.simd[rd] = out & simd_vector_mask(instr.size as usize);
        }
        Opcode::SimdUshr => {
            let element_size = instr.cond.max(1) as usize;
            let shift = instr.imm as u32;
            let bits = (element_size * 8) as u32;
            let lanes = (instr.size as usize / element_size).max(1);
            let element_mask = simd_element_mask(element_size);
            let mut out = 0u128;
            for lane in 0..lanes {
                let value = simd_element(cpu.simd[rn], lane, element_size);
                let shifted = if shift >= bits { 0 } else { value >> shift };
                out |= (shifted & element_mask) << (lane * bits as usize);
            }
            cpu.simd[rd] = out & simd_vector_mask(instr.size as usize);
        }
        Opcode::SimdUshl => {
            let element_size = instr.imm.max(1) as usize;
            let bits = element_size * 8;
            let lanes = instr.size as usize / element_size;
            let element_mask = simd_element_mask(element_size);
            let mut out = 0u128;
            for lane in 0..lanes {
                let value = simd_element(cpu.simd[rn], lane, element_size);
                let shift = simd_signed_element(cpu.simd[rm], lane, element_size);
                let shifted = if shift >= 0 {
                    let amount = shift as usize;
                    if amount >= bits { 0 } else { value << amount }
                } else {
                    let amount = shift.unsigned_abs() as usize;
                    if amount >= bits { 0 } else { value >> amount }
                };
                out |= (shifted & element_mask) << (lane * bits);
            }
            cpu.simd[rd] = out & simd_vector_mask(instr.size as usize);
        }
        Opcode::SimdXtn => {
            let dest_element_size = instr.imm.max(1) as usize;
            let src_element_size = dest_element_size * 2;
            let dest_mask = simd_element_mask(dest_element_size);
            let lanes = instr.size as usize / dest_element_size;
            let mut out = 0u128;
            for lane in 0..lanes {
                out |= (simd_element(cpu.simd[rn], lane, src_element_size) & dest_mask)
                    << (lane * dest_element_size * 8);
            }
            cpu.simd[rd] = out;
        }
        Opcode::SimdRev64 => {
            let element_size = instr.imm.max(1) as usize;
            cpu.simd[rd] =
                simd_reverse_elements_in_groups(cpu.simd[rn], element_size, instr.size as usize, 8);
        }
        Opcode::SimdRev32 => {
            let element_size = instr.imm.max(1) as usize;
            cpu.simd[rd] =
                simd_reverse_elements_in_groups(cpu.simd[rn], element_size, instr.size as usize, 4);
        }
        Opcode::SimdNot => {
            let lanes_mask = if instr.size == 8 {
                u64::MAX as u128
            } else {
                u128::MAX
            };
            cpu.simd[rd] = !cpu.simd[rn] & lanes_mask;
        }
        Opcode::SimdBit => {
            let dest = cpu.simd[rd];
            let src = cpu.simd[rn];
            let mask = cpu.simd[rm];
            cpu.simd[rd] = (dest & !mask) | (src & mask);
        }
        Opcode::SimdAnd => {
            cpu.simd[rd] = (cpu.simd[rn] & cpu.simd[rm]) & simd_vector_mask(instr.size as usize);
        }
        Opcode::SimdOrr => {
            cpu.simd[rd] = (cpu.simd[rn] | cpu.simd[rm]) & simd_vector_mask(instr.size as usize);
        }
        Opcode::SimdInsElem => {
            let element_size = instr.cond.max(1) as usize;
            let dest_lane = (instr.imm >> 8) as usize;
            let source_lane = (instr.imm & 0xff) as usize;
            let bits = element_size * 8;
            let element_mask = simd_element_mask(element_size);
            let element = simd_element(cpu.simd[rn], source_lane, element_size);
            let dest_mask = element_mask << (dest_lane * bits);
            cpu.simd[rd] =
                (cpu.simd[rd] & !dest_mask) | ((element & element_mask) << (dest_lane * bits));
        }
        Opcode::SimdUzp1 => {
            let element_size = instr.imm.max(1) as usize;
            let bits = element_size * 8;
            let lanes = instr.size as usize / element_size;
            let half = lanes / 2;
            let mut out = 0u128;
            for lane in 0..half {
                out |= simd_element(cpu.simd[rn], lane * 2, element_size) << (lane * bits);
                out |= simd_element(cpu.simd[rm], lane * 2, element_size) << ((lane + half) * bits);
            }
            cpu.simd[rd] = out & simd_vector_mask(instr.size as usize);
        }
        Opcode::SimdZip1 | Opcode::SimdZip2 => {
            let element_size = instr.imm.max(1) as usize;
            let high_half = instr.op == Opcode::SimdZip2;
            cpu.simd[rd] = simd_zip(
                cpu.simd[rn],
                cpu.simd[rm],
                element_size,
                instr.size as usize,
                high_half,
            );
        }
        Opcode::SimdTbl => {
            let table_count = instr.cond.max(1) as usize;
            let mut out = 0u128;
            for lane in 0..instr.size as usize {
                let index = simd_byte(cpu.simd[rm], lane) as usize;
                let byte = if index < table_count * 16 {
                    let table_reg = (rn + index / 16) % 32;
                    simd_byte(cpu.simd[table_reg], index % 16)
                } else {
                    0
                };
                out |= (byte as u128) << (lane * 8);
            }
            cpu.simd[rd] = out;
        }
        Opcode::SimdEor => {
            let value = cpu.simd[rn] ^ cpu.simd[rm];
            cpu.simd[rd] = if instr.size == 8 {
                value & u64::MAX as u128
            } else {
                value
            };
        }
        Opcode::SimdBicImm => {
            let element_size = instr.cond.max(1) as usize;
            let mask = simd_replicate_element(instr.imm as u128, element_size, instr.size as usize);
            let lanes_mask = if instr.size == 8 {
                u64::MAX as u128
            } else {
                u128::MAX
            };
            cpu.simd[rd] = (cpu.simd[rd] & !mask) & lanes_mask;
        }
        Opcode::SimdMvni => {
            let element_size = instr.cond.max(1) as usize;
            let bits = element_size * 8;
            let element_mask = if bits == 128 {
                u128::MAX
            } else {
                (1u128 << bits) - 1
            };
            let element = !(instr.imm as u128) & element_mask;
            cpu.simd[rd] = simd_replicate_element(element, element_size, instr.size as usize);
        }
        Opcode::SimdUshll => {
            let src_element_size = instr.cond.max(1) as usize;
            let dst_element_size = src_element_size * 2;
            let src_bits = src_element_size * 8;
            let dst_bits = dst_element_size * 8;
            let dst_mask = simd_element_mask(dst_element_size);
            let shift = instr.imm as usize;
            let lanes = 8 / src_element_size;
            let mut out = 0u128;
            for lane in 0..lanes {
                let src = simd_element(cpu.simd[rn], lane, src_element_size);
                let widened = if shift >= dst_bits {
                    0
                } else {
                    (src << shift) & dst_mask
                };
                out |= widened << (lane * src_bits * 2);
            }
            cpu.simd[rd] = out;
        }
        Opcode::SimdSshll => {
            let src_element_size = instr.cond.max(1) as usize;
            let dst_element_size = src_element_size * 2;
            let dst_bits = dst_element_size * 8;
            let dst_mask = simd_element_mask(dst_element_size);
            let shift = instr.imm as usize;
            let lanes = 8 / src_element_size;
            let mut out = 0u128;
            for lane in 0..lanes {
                let src = simd_signed_element(cpu.simd[rn], lane, src_element_size) as i128;
                let widened = if shift >= dst_bits {
                    0
                } else {
                    ((src << shift) as u128) & dst_mask
                };
                out |= widened << (lane * dst_bits);
            }
            cpu.simd[rd] = out;
        }
        Opcode::SimdShll => {
            let src_element_size = instr.cond.max(1) as usize;
            let dst_element_size = src_element_size * 2;
            let dst_bits = dst_element_size * 8;
            let dst_mask = simd_element_mask(dst_element_size);
            let shift = instr.imm as usize;
            let lanes = 8 / src_element_size;
            let source_base_lane = if instr.sf { lanes } else { 0 };
            let mut out = 0u128;
            for lane in 0..lanes {
                let src = simd_element(cpu.simd[rn], source_base_lane + lane, src_element_size);
                let widened = if shift >= dst_bits {
                    0
                } else {
                    (src << shift) & dst_mask
                };
                out |= widened << (lane * dst_bits);
            }
            cpu.simd[rd] = out;
        }
        Opcode::SimdFpNeg => {
            let element_size = instr.imm.max(1) as usize;
            let bits = element_size * 8;
            let sign_bit = 1u128 << (bits - 1);
            let lanes = instr.size as usize / element_size;
            let mut out = 0u128;
            for lane in 0..lanes {
                let value = simd_element(cpu.simd[rn], lane, element_size) ^ sign_bit;
                out |= value << (lane * bits);
            }
            cpu.simd[rd] = out;
        }
        _ => unreachable!(),
    }
}

pub(super) fn exec_fp_scalar(cpu: &mut Armv8Cpu, instr: Instr) {
    match instr.op {
        Opcode::FpAdd => exec_fp_binary(cpu, instr, |a, b| a + b, |a, b| a + b),
        Opcode::FpSub => exec_fp_binary(cpu, instr, |a, b| a - b, |a, b| a - b),
        Opcode::FpMul => exec_fp_binary(cpu, instr, |a, b| a * b, |a, b| a * b),
        Opcode::FpDiv => exec_fp_binary(cpu, instr, |a, b| a / b, |a, b| a / b),
        Opcode::Fmadd => exec_fp_fused(cpu, instr, false),
        Opcode::Fmsub => exec_fp_fused(cpu, instr, true),
        Opcode::Fnmsub => exec_fp_fnmsub(cpu, instr),
        Opcode::FpNeg => {
            let sign_mask = if instr.size == 4 {
                1u64 << 31
            } else {
                1u64 << 63
            };
            write_fp_bits(
                cpu,
                instr.rd,
                read_fp_bits(cpu, instr.rn, instr.size) ^ sign_mask,
                instr.size,
            );
        }
        Opcode::FpAbs => {
            let sign_mask = if instr.size == 4 {
                1u64 << 31
            } else {
                1u64 << 63
            };
            write_fp_bits(
                cpu,
                instr.rd,
                read_fp_bits(cpu, instr.rn, instr.size) & !sign_mask,
                instr.size,
            );
        }
        Opcode::FpSqrt => {
            if instr.size == 4 {
                let value = f32::from_bits(read_fp_bits(cpu, instr.rn, 4) as u32).sqrt();
                write_fp_bits(cpu, instr.rd, value.to_bits() as u64, 4);
            } else {
                let value = f64::from_bits(read_fp_bits(cpu, instr.rn, 8)).sqrt();
                write_fp_bits(cpu, instr.rd, value.to_bits(), 8);
            }
        }
        Opcode::FpFcvt => {
            let src_size = instr.cond;
            if src_size == 4 && instr.size == 8 {
                let value = f32::from_bits(read_fp_bits(cpu, instr.rn, 4) as u32) as f64;
                write_fp_bits(cpu, instr.rd, value.to_bits(), 8);
            } else if src_size == 8 && instr.size == 4 {
                let value = f64::from_bits(read_fp_bits(cpu, instr.rn, 8)) as f32;
                write_fp_bits(cpu, instr.rd, value.to_bits() as u64, 4);
            }
        }
        Opcode::FpFrintm => {
            if instr.size == 4 {
                let value = f32::from_bits(read_fp_bits(cpu, instr.rn, 4) as u32).floor();
                write_fp_bits(cpu, instr.rd, value.to_bits() as u64, 4);
            } else {
                let value = f64::from_bits(read_fp_bits(cpu, instr.rn, 8)).floor();
                write_fp_bits(cpu, instr.rd, value.to_bits(), 8);
            }
        }
        Opcode::FpMovImm => {
            write_fp_bits(
                cpu,
                instr.rd,
                fp_expand_imm(instr.imm as u8, instr.size),
                instr.size,
            );
        }
        Opcode::Scvtf => {
            let value = if instr.sf {
                read_reg(cpu, instr.rn, true) as i64 as f64
            } else {
                read_reg(cpu, instr.rn, false) as u32 as i32 as f64
            };
            let scaled = if instr.cond == 1 {
                value / 2f64.powi(instr.imm as i32)
            } else {
                value
            };
            if instr.size == 4 {
                write_fp_bits(cpu, instr.rd, (scaled as f32).to_bits() as u64, 4);
            } else {
                write_fp_bits(cpu, instr.rd, scaled.to_bits(), 8);
            }
        }
        Opcode::Ucvtf => {
            let value = if instr.sf {
                read_reg(cpu, instr.rn, true) as f64
            } else {
                read_reg(cpu, instr.rn, false) as u32 as f64
            };
            let scaled = if instr.cond == 1 {
                value / 2f64.powi(instr.imm as i32)
            } else {
                value
            };
            if instr.size == 4 {
                write_fp_bits(cpu, instr.rd, (scaled as f32).to_bits() as u64, 4);
            } else {
                write_fp_bits(cpu, instr.rd, scaled.to_bits(), 8);
            }
        }
        Opcode::Fcvtzs => {
            let value = read_fp_as_f64(cpu, instr.rn, instr.size).trunc();
            if instr.sf {
                write_reg(cpu, instr.rd, value as i64 as u64, true);
            } else {
                write_reg(cpu, instr.rd, value as i32 as u32 as u64, false);
            }
        }
        Opcode::Fcvtzu => {
            let value = read_fp_as_f64(cpu, instr.rn, instr.size).trunc();
            if instr.sf {
                write_reg(cpu, instr.rd, value as u64, true);
            } else {
                write_reg(cpu, instr.rd, value as u32 as u64, false);
            }
        }
        Opcode::Fcmp | Opcode::Fcmpe => {
            let lhs = read_fp_as_f64(cpu, instr.rn, instr.size);
            let rhs = if instr.cond == 1 {
                0.0
            } else {
                read_fp_as_f64(cpu, instr.rm, instr.size)
            };
            set_fp_compare_flags(cpu, lhs, rhs);
        }
        Opcode::Fcsel => {
            let src = if cond_taken(cpu, instr.cond) {
                instr.rn
            } else {
                instr.rm
            };
            write_fp_bits(
                cpu,
                instr.rd,
                read_fp_bits(cpu, src, instr.size),
                instr.size,
            );
        }
        _ => unreachable!(),
    }
}

fn exec_fp_fnmsub(cpu: &mut Armv8Cpu, instr: Instr) {
    if instr.size == 4 {
        let n = f32::from_bits(read_fp_bits(cpu, instr.rn, 4) as u32);
        let m = f32::from_bits(read_fp_bits(cpu, instr.rm, 4) as u32);
        let a = f32::from_bits(read_fp_bits(cpu, instr.cond, 4) as u32);
        write_fp_bits(cpu, instr.rd, n.mul_add(m, -a).to_bits() as u64, 4);
    } else {
        let n = f64::from_bits(read_fp_bits(cpu, instr.rn, 8));
        let m = f64::from_bits(read_fp_bits(cpu, instr.rm, 8));
        let a = f64::from_bits(read_fp_bits(cpu, instr.cond, 8));
        write_fp_bits(cpu, instr.rd, n.mul_add(m, -a).to_bits(), 8);
    }
}

fn exec_fp_fused(cpu: &mut Armv8Cpu, instr: Instr, subtract_product: bool) {
    if instr.size == 4 {
        let n = f32::from_bits(read_fp_bits(cpu, instr.rn, 4) as u32);
        let m = f32::from_bits(read_fp_bits(cpu, instr.rm, 4) as u32);
        let a = f32::from_bits(read_fp_bits(cpu, instr.cond, 4) as u32);
        let value = if subtract_product {
            (-n).mul_add(m, a)
        } else {
            n.mul_add(m, a)
        };
        write_fp_bits(cpu, instr.rd, value.to_bits() as u64, 4);
    } else {
        let n = f64::from_bits(read_fp_bits(cpu, instr.rn, 8));
        let m = f64::from_bits(read_fp_bits(cpu, instr.rm, 8));
        let a = f64::from_bits(read_fp_bits(cpu, instr.cond, 8));
        let value = if subtract_product {
            (-n).mul_add(m, a)
        } else {
            n.mul_add(m, a)
        };
        write_fp_bits(cpu, instr.rd, value.to_bits(), 8);
    }
}

fn exec_fp_binary<F32, F64>(cpu: &mut Armv8Cpu, instr: Instr, op32: F32, op64: F64)
where
    F32: FnOnce(f32, f32) -> f32,
    F64: FnOnce(f64, f64) -> f64,
{
    if instr.size == 4 {
        let lhs = f32::from_bits(read_fp_bits(cpu, instr.rn, 4) as u32);
        let rhs = f32::from_bits(read_fp_bits(cpu, instr.rm, 4) as u32);
        write_fp_bits(cpu, instr.rd, op32(lhs, rhs).to_bits() as u64, 4);
    } else {
        let lhs = f64::from_bits(read_fp_bits(cpu, instr.rn, 8));
        let rhs = f64::from_bits(read_fp_bits(cpu, instr.rm, 8));
        write_fp_bits(cpu, instr.rd, op64(lhs, rhs).to_bits(), 8);
    }
}

fn read_fp_bits(cpu: &Armv8Cpu, reg: u8, size: u8) -> u64 {
    if size == 4 {
        (cpu.simd[reg as usize] & u32::MAX as u128) as u64
    } else {
        cpu.simd[reg as usize] as u64
    }
}

fn write_fp_bits(cpu: &mut Armv8Cpu, reg: u8, bits: u64, size: u8) {
    cpu.simd[reg as usize] = if size == 4 {
        (bits as u32) as u128
    } else {
        bits as u128
    };
}

fn read_fp_as_f64(cpu: &Armv8Cpu, reg: u8, size: u8) -> f64 {
    if size == 4 {
        f32::from_bits(read_fp_bits(cpu, reg, 4) as u32) as f64
    } else {
        f64::from_bits(read_fp_bits(cpu, reg, 8))
    }
}

fn set_fp_compare_flags(cpu: &mut Armv8Cpu, lhs: f64, rhs: f64) {
    if lhs.is_nan() || rhs.is_nan() {
        cpu.pstate.set_nzcv(false, false, true, true);
    } else if lhs == rhs {
        cpu.pstate.set_nzcv(false, true, true, false);
    } else if lhs < rhs {
        cpu.pstate.set_nzcv(true, false, false, false);
    } else {
        cpu.pstate.set_nzcv(false, false, true, false);
    }
}

fn fp_expand_imm(imm8: u8, size: u8) -> u64 {
    let sign = (imm8 >> 7) as u64;
    let b = ((imm8 >> 6) & 1) as u64;
    let c = ((imm8 >> 5) & 1) as u64;
    let d = ((imm8 >> 4) & 1) as u64;
    let fraction = (imm8 & 0xF) as u64;

    if size == 4 {
        let exponent = ((!b & 1) << 7) | ((if b == 1 { 0x1F } else { 0 }) << 2) | (c << 1) | d;
        (sign << 31) | (exponent << 23) | (fraction << 19)
    } else {
        let exponent = ((!b & 1) << 10) | ((if b == 1 { 0xFF } else { 0 }) << 2) | (c << 1) | d;
        (sign << 63) | (exponent << 52) | (fraction << 48)
    }
}

fn simd_vector_mask(vector_size: usize) -> u128 {
    match vector_size {
        0 => 0,
        16.. => u128::MAX,
        bytes => (1u128 << (bytes * 8)) - 1,
    }
}

fn simd_element_mask(element_size: usize) -> u128 {
    let bits = element_size * 8;
    if bits >= 128 {
        u128::MAX
    } else {
        (1u128 << bits) - 1
    }
}

fn simd_byte(value: u128, lane: usize) -> u8 {
    ((value >> (lane * 8)) & 0xff) as u8
}

fn simd_element(value: u128, lane: usize, element_size: usize) -> u128 {
    let shift = lane * element_size * 8;
    (value >> shift) & simd_element_mask(element_size)
}

fn simd_reverse_elements_in_groups(
    value: u128,
    element_size: usize,
    vector_size: usize,
    group_size: usize,
) -> u128 {
    let elements_per_group = group_size / element_size;
    let groups = vector_size / group_size;
    let mut out = 0u128;
    for group in 0..groups {
        for index in 0..elements_per_group {
            let dst_lane = group * elements_per_group + index;
            let src_lane = group * elements_per_group + (elements_per_group - 1 - index);
            let element = simd_element(value, src_lane, element_size);
            out |= element << (dst_lane * element_size * 8);
        }
    }
    out & simd_vector_mask(vector_size)
}

fn simd_signed_element(value: u128, lane: usize, element_size: usize) -> i64 {
    let bits = element_size * 8;
    let raw = simd_element(value, lane, element_size);
    if bits == 64 {
        raw as u64 as i64
    } else {
        let sign = 1u128 << (bits - 1);
        let extended = if (raw & sign) != 0 {
            raw | (!0u128 << bits)
        } else {
            raw
        };
        extended as i128 as i64
    }
}

pub(super) fn simd_replicate_element(
    element: u128,
    element_size: usize,
    vector_size: usize,
) -> u128 {
    let bits = element_size * 8;
    let element_mask = if bits == 128 {
        u128::MAX
    } else {
        (1u128 << bits) - 1
    };
    let lanes = vector_size / element_size;
    let mut value = 0u128;
    for lane in 0..lanes {
        value |= (element & element_mask) << (lane * bits);
    }
    value
}

fn simd_elementwise_binary<F>(
    lhs: u128,
    rhs: u128,
    element_size: usize,
    vector_size: usize,
    f: F,
) -> u128
where
    F: Fn(u128, u128, u128) -> u128,
{
    let bits = element_size * 8;
    let element_mask = simd_element_mask(element_size);
    let lanes = vector_size / element_size;
    let mut out = 0u128;
    for lane in 0..lanes {
        let a = simd_element(lhs, lane, element_size);
        let b = simd_element(rhs, lane, element_size);
        out |= (f(a, b, element_mask) & element_mask) << (lane * bits);
    }
    out
}

fn simd_pairwise_binary<F>(
    lhs: u128,
    rhs: u128,
    element_size: usize,
    vector_size: usize,
    f: F,
) -> u128
where
    F: Fn(u128, u128, u128) -> u128,
{
    let bits = element_size * 8;
    let element_mask = if bits == 128 {
        u128::MAX
    } else {
        (1u128 << bits) - 1
    };
    let elements = vector_size / element_size;
    let pairs_per_source = elements / 2;
    let mut out = 0u128;
    for lane in 0..elements {
        let source = if lane < pairs_per_source { lhs } else { rhs };
        let pair = (lane % pairs_per_source) * 2;
        let a = simd_element(source, pair, element_size);
        let b = simd_element(source, pair + 1, element_size);
        out |= (f(a, b, element_mask) & element_mask) << (lane * bits);
    }
    out
}

fn simd_zip(
    lhs: u128,
    rhs: u128,
    element_size: usize,
    vector_size: usize,
    high_half: bool,
) -> u128 {
    let bits = element_size * 8;
    let lanes = vector_size / element_size;
    let half = lanes / 2;
    let start = if high_half { half } else { 0 };
    let mut out = 0u128;
    for lane in 0..half {
        out |= simd_element(lhs, start + lane, element_size) << ((lane * 2) * bits);
        out |= simd_element(rhs, start + lane, element_size) << ((lane * 2 + 1) * bits);
    }
    out & simd_vector_mask(vector_size)
}

fn simd_compare_elements_with_zero(value: u128, element_size: usize, vector_size: usize) -> u128 {
    let bits = element_size * 8;
    let element_mask = if bits == 128 {
        u128::MAX
    } else {
        (1u128 << bits) - 1
    };
    let lanes = vector_size / element_size;
    let mut out = 0u128;
    for lane in 0..lanes {
        if simd_element(value, lane, element_size) == 0 {
            out |= element_mask << (lane * bits);
        }
    }
    out
}

fn simd_compare_vec_bytes<F>(lhs: u128, rhs: u128, pred: F) -> u128
where
    F: Fn(u8, u8) -> bool,
{
    let mut out = 0u128;
    for lane in 0..16 {
        let byte = if pred(simd_byte(lhs, lane), simd_byte(rhs, lane)) {
            0xffu8
        } else {
            0
        };
        out |= (byte as u128) << (lane * 8);
    }
    out
}

// ── Register extension & shifting ──

pub(super) fn extend_reg_val(cpu: &Armv8Cpu, rm: u8, option: u8, shift: u8, sf: bool) -> u64 {
    let mut val = read_reg(
        cpu,
        rm,
        if option == 3 || option == 7 {
            sf
        } else {
            option >= 2
        },
    );
    val = match option {
        0 => (val as u8) as u64,           // UXTB
        1 => (val as u16) as u64,          // UXTH
        2 => (val as u32) as u64,          // UXTW
        3 => val,                          // UXTX
        4 => ((val as i8) as i64) as u64,  // SXTB
        5 => ((val as i16) as i64) as u64, // SXTH
        6 => ((val as i32) as i64) as u64, // SXTW
        7 => val,                          // SXTX
        _ => val,
    };
    if sf {
        val << shift
    } else {
        ((val as u32) << shift) as u64
    }
}

pub(super) fn shifted_reg_val(cpu: &Armv8Cpu, rm: u8, shift_type: u8, amount: u8, sf: bool) -> u64 {
    let val = read_reg(cpu, rm, sf);
    let amount = amount as u32;
    if amount == 0 {
        return val;
    }
    match shift_type {
        0 => {
            if sf {
                val << amount
            } else {
                ((val as u32) << amount) as u64
            }
        }
        1 => {
            if sf {
                val >> amount
            } else {
                ((val as u32) >> amount) as u64
            }
        }
        2 => {
            if sf {
                ((val as i64) >> amount) as u64
            } else {
                (((val as u32) as i32) >> amount) as u64
            }
        }
        3 => {
            if sf {
                val.rotate_right(amount)
            } else {
                (val as u32).rotate_right(amount) as u64
            }
        }
        _ => val,
    }
}

// ── Logical register ──

pub(super) fn exec_logical_reg(cpu: &mut Armv8Cpu, instr: Instr) {
    let n = (instr.cond & 4) != 0;
    let shift_type = instr.cond & 3;
    let mut rhs = shifted_reg_val(cpu, instr.rm, shift_type, instr.imm as u8, instr.sf);
    if n {
        rhs = !rhs;
        if !instr.sf {
            rhs &= 0xFFFFFFFF;
        }
    }
    let lhs = read_reg(cpu, instr.rn, instr.sf);
    let val = match instr.op {
        Opcode::AndReg => lhs & rhs,
        Opcode::OrrReg => lhs | rhs,
        Opcode::EorReg => lhs ^ rhs,
        Opcode::AndsReg => {
            set_nz_flags(cpu, lhs & rhs, instr.sf);
            lhs & rhs
        }
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
        Opcode::Bfm => {
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
        if signed {
            sign_extend(extracted, s - r, size)
        } else {
            extracted
        }
    } else {
        let len = s + 1;
        let mask = bitmask(len);
        let shift = size - r;
        let extracted = (src & mask) << shift;
        if signed {
            sign_extend(extracted, shift + s, size)
        } else {
            extracted
        }
    };
    word_truncate(result, size)
}

fn bitfield_insert(dst: u64, src: u64, r: u32, s: u32, size: u32) -> u64 {
    let result = if s >= r {
        let len = s - r + 1;
        let mask = bitmask(len);
        let extracted = (src >> r) & mask;
        (dst & !mask) | extracted
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
        0 => {
            if instr.sf {
                a.wrapping_add(n.wrapping_mul(m))
            } else {
                ((a as u32).wrapping_add((n as u32).wrapping_mul(m as u32))) as u64
            }
        }
        1 => a.wrapping_add((n as u32 as u64).wrapping_mul(m as u32 as u64)),
        2 => {
            a.wrapping_add(((n as u32 as i32) as i64).wrapping_mul((m as u32 as i32) as i64) as u64)
        }
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
        0 => {
            if instr.sf {
                a.wrapping_sub(n.wrapping_mul(m))
            } else {
                ((a as u32).wrapping_sub((n as u32).wrapping_mul(m as u32))) as u64
            }
        }
        1 => a.wrapping_sub((n as u32 as u64).wrapping_mul(m as u32 as u64)),
        2 => {
            a.wrapping_sub(((n as u32 as i32) as i64).wrapping_mul((m as u32 as i32) as i64) as u64)
        }
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

pub(super) fn exec_extract(cpu: &mut Armv8Cpu, instr: Instr) {
    let size = if instr.sf { 64 } else { 32 };
    let lsb = (instr.imm as u32) & (size - 1);
    let low = read_reg(cpu, instr.rm, instr.sf);
    let high = read_reg(cpu, instr.rn, instr.sf);
    let val = if lsb == 0 {
        low
    } else if instr.sf {
        (low >> lsb) | (high << (64 - lsb))
    } else {
        (((low as u32) >> lsb) | ((high as u32) << (32 - lsb))) as u64
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
        if signed {
            (n as i64).checked_div(m as i64).unwrap_or(n as i64) as u64
        } else {
            n / m
        }
    } else if signed {
        (n as i32).checked_div(m as i32).unwrap_or(n as i32) as u32 as u64
    } else {
        ((n as u32) / (m as u32)) as u64
    };
    write_reg(cpu, instr.rd, val, instr.sf);
}

// ── Reverse bits/bytes ──

pub(super) fn exec_rev(cpu: &mut Armv8Cpu, instr: Instr) {
    if instr.sf {
        write_reg(
            cpu,
            instr.rd,
            read_reg(cpu, instr.rn, true).swap_bytes(),
            true,
        );
    } else {
        write_reg(
            cpu,
            instr.rd,
            (read_reg(cpu, instr.rn, false) as u32).swap_bytes() as u64,
            false,
        );
    }
}

pub(super) fn exec_rev16(cpu: &mut Armv8Cpu, instr: Instr) {
    const MASK_EVEN: u64 = 0xFF00_FF00_FF00_FF00;
    const MASK_ODD: u64 = 0x00FF_00FF_00FF_00FF;
    if instr.sf {
        let val = read_reg(cpu, instr.rn, true);
        write_reg(
            cpu,
            instr.rd,
            ((val & MASK_EVEN) >> 8) | ((val & MASK_ODD) << 8),
            true,
        );
    } else {
        let val = read_reg(cpu, instr.rn, false) as u32;
        write_reg(
            cpu,
            instr.rd,
            (((val & 0xFF00_FF00) >> 8) | ((val & 0x00FF_00FF) << 8)) as u64,
            false,
        );
    }
}

pub(super) fn exec_rbit(cpu: &mut Armv8Cpu, instr: Instr) {
    if instr.sf {
        write_reg(
            cpu,
            instr.rd,
            read_reg(cpu, instr.rn, true).reverse_bits(),
            true,
        );
    } else {
        write_reg(
            cpu,
            instr.rd,
            (read_reg(cpu, instr.rn, false) as u32).reverse_bits() as u64,
            false,
        );
    }
}

pub(super) fn exec_clz(cpu: &mut Armv8Cpu, instr: Instr) {
    if instr.sf {
        write_reg(
            cpu,
            instr.rd,
            read_reg(cpu, instr.rn, true).leading_zeros() as u64,
            true,
        );
    } else {
        write_reg(
            cpu,
            instr.rd,
            (read_reg(cpu, instr.rn, false) as u32).leading_zeros() as u64,
            false,
        );
    }
}

pub(super) fn exec_crc32(cpu: &mut Armv8Cpu, instr: Instr) {
    let mut crc = read_reg(cpu, instr.rn, false) as u32;
    let value = read_reg(cpu, instr.rm, instr.size == 8);

    for byte_index in 0..instr.size {
        crc ^= ((value >> (byte_index * 8)) & 0xff) as u32;
        for _ in 0..8 {
            crc = if (crc & 1) != 0 {
                (crc >> 1) ^ 0xedb8_8320
            } else {
                crc >> 1
            };
        }
    }

    write_reg(cpu, instr.rd, crc as u64, false);
}
