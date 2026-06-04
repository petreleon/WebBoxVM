use super::*;

pub(in crate::arm64::execute) fn exec_simd_shift(cpu: &mut Armv8Cpu, instr: Instr) {
    let rd = instr.rd as usize;
    let rn = instr.rn as usize;
    let rm = instr.rm as usize;

    match instr.op {
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
        Opcode::SimdSri => {
            let element_size = instr.cond.max(1) as usize;
            let bits = element_size * 8;
            let shift = instr.imm as usize;
            let lanes = (instr.size as usize / element_size).max(1);
            let element_mask = simd_element_mask(element_size);
            let insert_mask = if shift >= bits {
                0
            } else {
                element_mask >> shift
            };
            let mut out = cpu.simd[rd];
            for lane in 0..lanes {
                let source = simd_element(cpu.simd[rn], lane, element_size);
                let dest = simd_element(cpu.simd[rd], lane, element_size);
                let inserted = if shift >= bits { 0 } else { source >> shift };
                let value = (inserted & insert_mask) | (dest & !insert_mask & element_mask);
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
        Opcode::SimdSshr => {
            let element_size = instr.cond.max(1) as usize;
            let shift = instr.imm as u32;
            let bits = (element_size * 8) as u32;
            let lanes = (instr.size as usize / element_size).max(1);
            let element_mask = simd_element_mask(element_size);
            let mut out = 0u128;
            for lane in 0..lanes {
                let value = simd_signed_element(cpu.simd[rn], lane, element_size);
                let shifted = if shift >= bits {
                    if value < 0 { -1i128 } else { 0 }
                } else {
                    (value >> shift) as i128
                };
                out |= ((shifted as u128) & element_mask) << (lane * bits as usize);
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
                let shift = simd_element(cpu.simd[rm], lane, element_size) as u8 as i8;
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
        Opcode::SimdSshl => {
            let element_size = instr.imm.max(1) as usize;
            let bits = element_size * 8;
            let lanes = instr.size as usize / element_size;
            let element_mask = simd_element_mask(element_size);
            let mut out = 0u128;
            for lane in 0..lanes {
                let value = simd_signed_element(cpu.simd[rn], lane, element_size) as i128;
                let shift = simd_element(cpu.simd[rm], lane, element_size) as u8 as i8;
                let shifted = if shift >= 0 {
                    let amount = shift as usize;
                    if amount >= bits {
                        0
                    } else {
                        (value as u128) << amount
                    }
                } else {
                    let amount = shift.unsigned_abs() as usize;
                    if amount >= bits {
                        if value < 0 { element_mask } else { 0 }
                    } else {
                        (value >> amount) as u128
                    }
                };
                out |= (shifted & element_mask) << (lane * bits);
            }
            cpu.simd[rd] = out & simd_vector_mask(instr.size as usize);
        }
        _ => unreachable!(),
    }
}
