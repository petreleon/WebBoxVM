use super::*;

pub(in crate::arm64::execute) fn exec_simd_integer(cpu: &mut Armv8Cpu, instr: Instr) {
    let rd = instr.rd as usize;
    let rn = instr.rn as usize;
    let rm = instr.rm as usize;

    match instr.op {
        Opcode::SimdShrn => {
            let src = cpu.simd[rn];
            let shift = instr.imm as usize;
            let dest_element_size = instr.cond.max(1) as usize;
            let src_element_size = dest_element_size * 2;
            let dest_bits = dest_element_size * 8;
            let src_bits = src_element_size * 8;
            let dest_mask = simd_element_mask(dest_element_size);
            let lanes = (instr.size as usize / dest_element_size).max(1);
            let mut out = 0u128;
            for lane in 0..lanes {
                let value = simd_element(src, lane, src_element_size);
                let shifted = if shift >= src_bits { 0 } else { value >> shift };
                out |= (shifted & dest_mask) << (lane * dest_bits);
            }
            cpu.simd[rd] = out & simd_vector_mask(instr.size as usize);
        }
        Opcode::SimdAddhn | Opcode::SimdSubhn => {
            cpu.simd[rd] =
                simd_narrow_high(cpu.simd[rn], cpu.simd[rm], instr.imm as usize, |a, b| {
                    if instr.op == Opcode::SimdAddhn {
                        a.wrapping_add(b)
                    } else {
                        a.wrapping_sub(b)
                    }
                });
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
        Opcode::SimdMulVec => {
            cpu.simd[rd] = simd_elementwise_binary(
                cpu.simd[rn],
                cpu.simd[rm],
                instr.imm.max(1) as usize,
                instr.size as usize,
                |a, b, mask| a.wrapping_mul(b) & mask,
            );
        }
        Opcode::SimdMlaVec => {
            let accumulator = cpu.simd[rd];
            let lhs = cpu.simd[rn];
            let rhs = cpu.simd[rm];
            let element_size = instr.imm.max(1) as usize;
            let element_mask = simd_element_mask(element_size);
            cpu.simd[rd] = simd_elementwise_ternary(
                accumulator,
                lhs,
                rhs,
                element_size,
                instr.size as usize,
                |acc, a, b| acc.wrapping_add(a.wrapping_mul(b)) & element_mask,
            );
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
        _ => unreachable!(),
    }
}

fn simd_narrow_high<F>(lhs: u128, rhs: u128, dest_element_size: usize, op: F) -> u128
where
    F: Fn(u128, u128) -> u128,
{
    let src_element_size = dest_element_size * 2;
    let dest_bits = dest_element_size * 8;
    let dest_mask = simd_element_mask(dest_element_size);
    let src_mask = simd_element_mask(src_element_size);
    let lanes = 8 / dest_element_size;
    let mut out = 0u128;
    for lane in 0..lanes {
        let value = op(
            simd_element(lhs, lane, src_element_size),
            simd_element(rhs, lane, src_element_size),
        ) & src_mask;
        out |= ((value >> dest_bits) & dest_mask) << (lane * dest_bits);
    }
    out & simd_vector_mask(lanes * dest_element_size)
}
