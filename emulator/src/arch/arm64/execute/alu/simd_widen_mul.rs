use super::*;

pub(in crate::arch::arm64::execute) fn exec_simd_widen_mul(cpu: &mut Armv8Cpu, instr: Instr) {
    let src_size = instr.cond.max(1) as usize;
    let dst_size = src_size * 2;
    let dst_bits = dst_size * 8;
    let dst_mask = simd_element_mask(dst_size);
    let lanes = 8 / src_size;
    let source_base_lane = if instr.sf { lanes } else { 0 };
    let rd = instr.rd as usize;
    let rn = instr.rn as usize;
    let rm = instr.rm as usize;
    let mut out = if is_umull(instr.op) { 0 } else { cpu.simd[rd] };

    for lane in 0..lanes {
        let lhs = simd_element(cpu.simd[rn], source_base_lane + lane, src_size);
        let rhs = mul_rhs(cpu, instr, lane, source_base_lane, src_size, rm);
        let accum = if is_umull(instr.op) {
            0
        } else {
            simd_element(cpu.simd[rd], lane, dst_size)
        };
        let product = lhs.wrapping_mul(rhs) & dst_mask;
        let value = if instr.op == Opcode::SimdUmlsl {
            accum.wrapping_sub(product)
        } else {
            accum.wrapping_add(product)
        } & dst_mask;
        out &= !(dst_mask << (lane * dst_bits));
        out |= value << (lane * dst_bits);
    }

    cpu.simd[rd] = out & simd_vector_mask(instr.size as usize);
}

fn mul_rhs(
    cpu: &Armv8Cpu,
    instr: Instr,
    lane: usize,
    source_base_lane: usize,
    src_size: usize,
    rm: usize,
) -> u128 {
    if matches!(
        instr.op,
        Opcode::SimdUmlal | Opcode::SimdUmlsl | Opcode::SimdUmullElem
    ) {
        simd_element(cpu.simd[rm], instr.imm as usize, src_size)
    } else {
        simd_element(cpu.simd[rm], source_base_lane + lane, src_size)
    }
}

fn is_umull(op: Opcode) -> bool {
    matches!(op, Opcode::SimdUmull | Opcode::SimdUmullElem)
}
