use super::*;

pub(in crate::arm64::execute) fn exec_simd_fp_fused(cpu: &mut Armv8Cpu, instr: Instr) {
    let rd = instr.rd as usize;
    let rn = instr.rn as usize;
    let rm = instr.rm as usize;
    let subtract = instr.op == Opcode::SimdFpFmlsVec;
    cpu.simd[rd] = simd_fp_fused(
        cpu.simd[rd],
        cpu.simd[rn],
        cpu.simd[rm],
        instr.imm.max(1) as usize,
        instr.size as usize,
        subtract,
    );
}

fn simd_fp_fused(
    addend: u128,
    left: u128,
    right: u128,
    element_size: usize,
    vector_size: usize,
    subtract: bool,
) -> u128 {
    match element_size {
        4 => simd_fp_fused_f32(addend, left, right, vector_size, subtract),
        8 => simd_fp_fused_f64(addend, left, right, vector_size, subtract),
        _ => 0,
    }
}

fn simd_fp_fused_f32(
    addend: u128,
    left: u128,
    right: u128,
    vector_size: usize,
    subtract: bool,
) -> u128 {
    let mut out = 0u128;
    for lane in 0..vector_size / 4 {
        let acc = f32::from_bits(simd_element(addend, lane, 4) as u32);
        let lhs = f32::from_bits(simd_element(left, lane, 4) as u32);
        let rhs = f32::from_bits(simd_element(right, lane, 4) as u32);
        let lhs = if subtract { -lhs } else { lhs };
        out |= (lhs.mul_add(rhs, acc).to_bits() as u128) << (lane * 32);
    }
    out & simd_vector_mask(vector_size)
}

fn simd_fp_fused_f64(
    addend: u128,
    left: u128,
    right: u128,
    vector_size: usize,
    subtract: bool,
) -> u128 {
    let mut out = 0u128;
    for lane in 0..vector_size / 8 {
        let acc = f64::from_bits(simd_element(addend, lane, 8) as u64);
        let lhs = f64::from_bits(simd_element(left, lane, 8) as u64);
        let rhs = f64::from_bits(simd_element(right, lane, 8) as u64);
        let lhs = if subtract { -lhs } else { lhs };
        out |= (lhs.mul_add(rhs, acc).to_bits() as u128) << (lane * 64);
    }
    out & simd_vector_mask(vector_size)
}
