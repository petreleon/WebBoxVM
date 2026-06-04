use super::*;

pub(in crate::arm64::execute) fn exec_simd_shift_accumulate(cpu: &mut Armv8Cpu, instr: Instr) {
    let rd = instr.rd as usize;
    let rn = instr.rn as usize;
    let element_size = instr.cond.max(1) as usize;
    let bits = element_size * 8;
    let shift = instr.imm as usize;
    let lanes = (instr.size as usize / element_size).max(1);
    let element_mask = simd_element_mask(element_size);
    let mut out = 0u128;

    for lane in 0..lanes {
        let dest = simd_element(cpu.simd[rd], lane, element_size);
        let source = simd_element(cpu.simd[rn], lane, element_size);
        let shifted = if shift >= bits { 0 } else { source >> shift };
        let value = dest.wrapping_add(shifted) & element_mask;
        out |= value << (lane * bits);
    }

    cpu.simd[rd] = out & simd_vector_mask(instr.size as usize);
}
