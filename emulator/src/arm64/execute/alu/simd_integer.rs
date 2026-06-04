use super::*;

pub(in crate::arm64::execute) fn exec_simd_integer(cpu: &mut Armv8Cpu, instr: Instr) {
    let rd = instr.rd as usize;
    let rn = instr.rn as usize;
    let rm = instr.rm as usize;

    match instr.op {
        Opcode::SimdShrn | Opcode::SimdShrn2 | Opcode::SimdRshrn | Opcode::SimdRshrn2 => {
            let src = cpu.simd[rn];
            let shift = instr.imm as usize;
            let dest_element_size = instr.cond.max(1) as usize;
            let src_element_size = dest_element_size * 2;
            let dest_bits = dest_element_size * 8;
            let src_bits = src_element_size * 8;
            let dest_mask = simd_element_mask(dest_element_size);
            let lanes = (8 / dest_element_size).max(1);
            let mut out = 0u128;
            let round = matches!(instr.op, Opcode::SimdRshrn | Opcode::SimdRshrn2);
            for lane in 0..lanes {
                let value = simd_element(src, lane, src_element_size);
                let shifted = if shift >= src_bits {
                    0
                } else {
                    rounded_shift(value as i128, shift, round)
                };
                out |= (shifted & dest_mask) << (lane * dest_bits);
            }
            let high = matches!(instr.op, Opcode::SimdShrn2 | Opcode::SimdRshrn2);
            cpu.simd[rd] = place_narrow_part(cpu.simd[rd], out, high);
        }
        Opcode::SimdAddhn
        | Opcode::SimdAddhn2
        | Opcode::SimdRaddhn
        | Opcode::SimdRaddhn2
        | Opcode::SimdSubhn
        | Opcode::SimdSubhn2
        | Opcode::SimdRsubhn
        | Opcode::SimdRsubhn2 => {
            let add = matches!(
                instr.op,
                Opcode::SimdAddhn | Opcode::SimdAddhn2 | Opcode::SimdRaddhn | Opcode::SimdRaddhn2
            );
            let round = matches!(
                instr.op,
                Opcode::SimdRaddhn | Opcode::SimdRaddhn2 | Opcode::SimdRsubhn | Opcode::SimdRsubhn2
            );
            let narrow = simd_narrow_high(
                cpu.simd[rn],
                cpu.simd[rm],
                instr.imm as usize,
                round,
                |a, b| {
                    if add {
                        a as i128 + b as i128
                    } else {
                        a as i128 - b as i128
                    }
                },
            );
            let high = matches!(
                instr.op,
                Opcode::SimdAddhn2 | Opcode::SimdRaddhn2 | Opcode::SimdSubhn2 | Opcode::SimdRsubhn2
            );
            cpu.simd[rd] = place_narrow_part(cpu.simd[rd], narrow, high);
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
        Opcode::SimdXtn | Opcode::SimdXtn2 => {
            let dest_element_size = instr.imm.max(1) as usize;
            let src_element_size = dest_element_size * 2;
            let dest_mask = simd_element_mask(dest_element_size);
            let lanes = 8 / dest_element_size;
            let mut out = 0u128;
            for lane in 0..lanes {
                out |= (simd_element(cpu.simd[rn], lane, src_element_size) & dest_mask)
                    << (lane * dest_element_size * 8);
            }
            cpu.simd[rd] = place_narrow_part(cpu.simd[rd], out, instr.op == Opcode::SimdXtn2);
        }
        _ => unreachable!(),
    }
}

fn place_narrow_part(old: u128, narrow: u128, high: bool) -> u128 {
    let narrow = narrow & u64::MAX as u128;
    if high {
        (old & u64::MAX as u128) | (narrow << 64)
    } else {
        narrow
    }
}

fn rounded_shift(value: i128, shift: usize, round: bool) -> u128 {
    (if round {
        (value + (1i128 << (shift - 1))) >> shift
    } else {
        value >> shift
    }) as u128
}

fn simd_narrow_high<F>(lhs: u128, rhs: u128, dest_element_size: usize, round: bool, op: F) -> u128
where
    F: Fn(u128, u128) -> i128,
{
    let src_element_size = dest_element_size * 2;
    let dest_bits = dest_element_size * 8;
    let dest_mask = simd_element_mask(dest_element_size);
    let lanes = 8 / dest_element_size;
    let mut out = 0u128;
    for lane in 0..lanes {
        let value = op(
            simd_element(lhs, lane, src_element_size),
            simd_element(rhs, lane, src_element_size),
        );
        out |= (rounded_shift(value, dest_bits, round) & dest_mask) << (lane * dest_bits);
    }
    out & simd_vector_mask(lanes * dest_element_size)
}
