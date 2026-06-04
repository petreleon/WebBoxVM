use super::*;

pub(in crate::arm64::execute) fn is_simd_widen_opcode(op: Opcode) -> bool {
    matches!(
        op,
        Opcode::SimdUshll
            | Opcode::SimdSshll
            | Opcode::SimdShll
            | Opcode::SimdSaddl
            | Opcode::SimdUaddl
            | Opcode::SimdUsubl
            | Opcode::SimdSsubw
            | Opcode::SimdSaddw
            | Opcode::SimdUaddw
            | Opcode::SimdUmlal
            | Opcode::SimdUmlalVec
            | Opcode::SimdUmull
            | Opcode::SimdUmullElem
    )
}

pub(in crate::arm64::execute) fn exec_simd_widen(cpu: &mut Armv8Cpu, instr: Instr) {
    let rd = instr.rd as usize;
    let rn = instr.rn as usize;

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
        Opcode::SimdSaddl => exec_widen_add_sub(cpu, instr, false, true, false),
        Opcode::SimdUaddl => exec_widen_add_sub(cpu, instr, false, false, false),
        Opcode::SimdUsubl => exec_widen_add_sub(cpu, instr, false, false, true),
        Opcode::SimdSsubw => exec_widen_add_sub(cpu, instr, true, true, true),
        Opcode::SimdSaddw => exec_widen_add_sub(cpu, instr, true, true, false),
        Opcode::SimdUaddw => exec_widen_add_sub(cpu, instr, true, false, false),
        Opcode::SimdUmlal | Opcode::SimdUmlalVec | Opcode::SimdUmull | Opcode::SimdUmullElem => {
            exec_simd_widen_mul(cpu, instr)
        }
        _ => unreachable!(),
    }
}

fn exec_widen_add_sub(
    cpu: &mut Armv8Cpu,
    instr: Instr,
    lhs_wide: bool,
    signed: bool,
    subtract: bool,
) {
    let rd = instr.rd as usize;
    let rn = instr.rn as usize;
    let rm = instr.rm as usize;
    let src_element_size = instr.cond.max(1) as usize;
    let dst_element_size = src_element_size * 2;
    let dst_bits = dst_element_size * 8;
    let dst_mask = simd_element_mask(dst_element_size);
    let lanes = 8 / src_element_size;
    let source_base_lane = if instr.sf { lanes } else { 0 };
    let mut out = 0u128;

    for lane in 0..lanes {
        let lhs_lane = if lhs_wide {
            lane
        } else {
            source_base_lane + lane
        };
        let lhs_size = if lhs_wide {
            dst_element_size
        } else {
            src_element_size
        };
        let lhs = widen_arith_operand(cpu.simd[rn], lhs_lane, lhs_size, signed);
        let rhs = widen_arith_operand(
            cpu.simd[rm],
            source_base_lane + lane,
            src_element_size,
            signed,
        );
        let value = if subtract { lhs - rhs } else { lhs + rhs };
        out |= ((value as u128) & dst_mask) << (lane * dst_bits);
    }

    cpu.simd[rd] = out;
}

fn widen_arith_operand(value: u128, lane: usize, element_size: usize, signed: bool) -> i128 {
    if signed {
        simd_signed_element(value, lane, element_size) as i128
    } else {
        simd_element(value, lane, element_size) as i128
    }
}
