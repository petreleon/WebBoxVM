use super::*;

pub(in crate::arm64::execute) fn exec_simd_widen(cpu: &mut Armv8Cpu, instr: Instr) {
    let rd = instr.rd as usize;
    let rn = instr.rn as usize;
    let rm = instr.rm as usize;

    match instr.op {
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
        Opcode::SimdSaddl => {
            let src_element_size = instr.cond.max(1) as usize;
            let dst_element_size = src_element_size * 2;
            let dst_bits = dst_element_size * 8;
            let dst_mask = simd_element_mask(dst_element_size);
            let lanes = 8 / src_element_size;
            let source_base_lane = if instr.sf { lanes } else { 0 };
            let mut out = 0u128;
            for lane in 0..lanes {
                let lhs =
                    simd_signed_element(cpu.simd[rn], source_base_lane + lane, src_element_size)
                        as i128;
                let rhs =
                    simd_signed_element(cpu.simd[rm], source_base_lane + lane, src_element_size)
                        as i128;
                let value = ((lhs + rhs) as u128) & dst_mask;
                out |= value << (lane * dst_bits);
            }
            cpu.simd[rd] = out;
        }
        Opcode::SimdUsubl => {
            let src_element_size = instr.cond.max(1) as usize;
            let dst_element_size = src_element_size * 2;
            let dst_bits = dst_element_size * 8;
            let dst_mask = simd_element_mask(dst_element_size);
            let lanes = 8 / src_element_size;
            let source_base_lane = if instr.sf { lanes } else { 0 };
            let mut out = 0u128;
            for lane in 0..lanes {
                let lhs = simd_element(cpu.simd[rn], source_base_lane + lane, src_element_size);
                let rhs = simd_element(cpu.simd[rm], source_base_lane + lane, src_element_size);
                let value = lhs.wrapping_sub(rhs) & dst_mask;
                out |= value << (lane * dst_bits);
            }
            cpu.simd[rd] = out;
        }
        Opcode::SimdSsubw => {
            let src_element_size = instr.cond.max(1) as usize;
            let dst_element_size = src_element_size * 2;
            let dst_bits = dst_element_size * 8;
            let dst_mask = simd_element_mask(dst_element_size);
            let lanes = 8 / src_element_size;
            let source_base_lane = if instr.sf { lanes } else { 0 };
            let mut out = 0u128;
            for lane in 0..lanes {
                let lhs = simd_signed_element(cpu.simd[rn], lane, dst_element_size) as i128;
                let rhs =
                    simd_signed_element(cpu.simd[rm], source_base_lane + lane, src_element_size)
                        as i128;
                let value = ((lhs - rhs) as u128) & dst_mask;
                out |= value << (lane * dst_bits);
            }
            cpu.simd[rd] = out;
        }
        Opcode::SimdUmlal => {
            let src_element_size = instr.cond.max(1) as usize;
            let dst_element_size = src_element_size * 2;
            let dst_bits = dst_element_size * 8;
            let dst_mask = simd_element_mask(dst_element_size);
            let lanes = 8 / src_element_size;
            let source_base_lane = if instr.sf { lanes } else { 0 };
            let scalar = simd_element(cpu.simd[rm], instr.imm as usize, src_element_size);
            let mut out = cpu.simd[rd];
            for lane in 0..lanes {
                let lhs = simd_element(cpu.simd[rn], source_base_lane + lane, src_element_size);
                let accum = simd_element(cpu.simd[rd], lane, dst_element_size);
                let value = accum.wrapping_add(lhs.wrapping_mul(scalar)) & dst_mask;
                out &= !(dst_mask << (lane * dst_bits));
                out |= value << (lane * dst_bits);
            }
            cpu.simd[rd] = out & simd_vector_mask(instr.size as usize);
        }
        _ => unreachable!(),
    }
}
