use super::*;

pub(in crate::arm64::execute) fn exec_simd_logic(cpu: &mut Armv8Cpu, instr: Instr) {
    let rd = instr.rd as usize;
    let rn = instr.rn as usize;
    let rm = instr.rm as usize;

    match instr.op {
        Opcode::SimdNot => {
            let lanes_mask = if instr.size == 8 {
                u64::MAX as u128
            } else {
                u128::MAX
            };
            cpu.simd[rd] = !cpu.simd[rn] & lanes_mask;
        }
        Opcode::SimdRbit => {
            let mut out = 0u128;
            for lane in 0..instr.size as usize {
                let byte = simd_byte(cpu.simd[rn], lane).reverse_bits();
                out |= (byte as u128) << (lane * 8);
            }
            cpu.simd[rd] = out;
        }
        Opcode::SimdBsl => {
            let vector_mask = simd_vector_mask(instr.size as usize);
            let mask = cpu.simd[rd] & vector_mask;
            let src_true = cpu.simd[rn] & vector_mask;
            let src_false = cpu.simd[rm] & vector_mask;
            cpu.simd[rd] = ((src_true & mask) | (src_false & !mask)) & vector_mask;
        }
        Opcode::SimdBit => {
            let vector_mask = simd_vector_mask(instr.size as usize);
            let dest = cpu.simd[rd];
            let src = cpu.simd[rn];
            let mask = cpu.simd[rm];
            cpu.simd[rd] = ((dest & !mask) | (src & mask)) & vector_mask;
        }
        Opcode::SimdBif => {
            let vector_mask = simd_vector_mask(instr.size as usize);
            let dest = cpu.simd[rd];
            let src = cpu.simd[rn];
            let mask = cpu.simd[rm];
            cpu.simd[rd] = ((dest & mask) | (src & !mask)) & vector_mask;
        }
        Opcode::SimdAnd => {
            cpu.simd[rd] = (cpu.simd[rn] & cpu.simd[rm]) & simd_vector_mask(instr.size as usize);
        }
        Opcode::SimdBic => {
            cpu.simd[rd] = (cpu.simd[rn] & !cpu.simd[rm]) & simd_vector_mask(instr.size as usize);
        }
        Opcode::SimdOrr => {
            cpu.simd[rd] = (cpu.simd[rn] | cpu.simd[rm]) & simd_vector_mask(instr.size as usize);
        }
        Opcode::SimdOrn => {
            cpu.simd[rd] = (cpu.simd[rn] | !cpu.simd[rm]) & simd_vector_mask(instr.size as usize);
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
        _ => unreachable!(),
    }
}
