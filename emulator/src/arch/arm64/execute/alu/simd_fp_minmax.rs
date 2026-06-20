use super::*;

pub(in crate::arch::arm64::execute) fn exec_simd_fp_minmax(cpu: &mut Armv8Cpu, instr: Instr) {
    if is_pairwise(instr.op) {
        let element_size = instr.imm.max(1) as usize;
        if instr.size == instr.imm as u8 {
            exec_scalar_pairwise_minmax(cpu, instr, element_size);
            return;
        }
        cpu.simd[instr.rd as usize] = pairwise_minmax(
            cpu.simd[instr.rn as usize],
            cpu.simd[instr.rm as usize],
            element_size,
            instr.size as usize,
            instr.op,
        );
        return;
    }
    cpu.simd[instr.rd as usize] = simd_fp_elementwise_binary(
        cpu.simd[instr.rn as usize],
        cpu.simd[instr.rm as usize],
        instr.imm.max(1) as usize,
        instr.size as usize,
        |a, b| apply_f32(instr.op, a, b),
        |a, b| apply_f64(instr.op, a, b),
    );
}

fn apply_f32(op: Opcode, lhs: f32, rhs: f32) -> f32 {
    match op {
        Opcode::SimdFpFmaxVec => fp_max(lhs, rhs),
        Opcode::SimdFpFminVec => fp_min(lhs, rhs),
        Opcode::SimdFpFmaxnmVec => fp_max_num(lhs, rhs),
        Opcode::SimdFpFminnmVec => fp_min_num(lhs, rhs),
        Opcode::SimdFpFmaxp => fp_max(lhs, rhs),
        Opcode::SimdFpFminp => fp_min(lhs, rhs),
        Opcode::SimdFpFmaxnmp => fp_max_num(lhs, rhs),
        Opcode::SimdFpFminnmp => fp_min_num(lhs, rhs),
        _ => unreachable!(),
    }
}

fn apply_f64(op: Opcode, lhs: f64, rhs: f64) -> f64 {
    match op {
        Opcode::SimdFpFmaxVec => fp_max(lhs, rhs),
        Opcode::SimdFpFminVec => fp_min(lhs, rhs),
        Opcode::SimdFpFmaxnmVec => fp_max_num(lhs, rhs),
        Opcode::SimdFpFminnmVec => fp_min_num(lhs, rhs),
        Opcode::SimdFpFmaxp => fp_max(lhs, rhs),
        Opcode::SimdFpFminp => fp_min(lhs, rhs),
        Opcode::SimdFpFmaxnmp => fp_max_num(lhs, rhs),
        Opcode::SimdFpFminnmp => fp_min_num(lhs, rhs),
        _ => unreachable!(),
    }
}

fn is_pairwise(op: Opcode) -> bool {
    matches!(
        op,
        Opcode::SimdFpFmaxp | Opcode::SimdFpFminp | Opcode::SimdFpFmaxnmp | Opcode::SimdFpFminnmp
    )
}

fn exec_scalar_pairwise_minmax(cpu: &mut Armv8Cpu, instr: Instr, element_size: usize) {
    let src = cpu.simd[instr.rn as usize];
    let bits = minmax_fp_bits(
        simd_element(src, 0, element_size),
        simd_element(src, 1, element_size),
        element_size,
        instr.op,
    );
    write_fp_bits(cpu, instr.rd, bits as u64, instr.size);
}

fn pairwise_minmax(
    lhs: u128,
    rhs: u128,
    element_size: usize,
    vector_size: usize,
    op: Opcode,
) -> u128 {
    let lanes = vector_size / element_size;
    let mut out = 0u128;
    for lane in 0..lanes {
        out |= pairwise_lane(lhs, rhs, element_size, lanes, lane, op) << (lane * element_size * 8);
    }
    out & simd_vector_mask(vector_size)
}

fn pairwise_lane(
    lhs: u128,
    rhs: u128,
    element_size: usize,
    lanes: usize,
    lane: usize,
    op: Opcode,
) -> u128 {
    let a = pairwise_source(lhs, rhs, element_size, lanes, lane * 2);
    let b = pairwise_source(lhs, rhs, element_size, lanes, lane * 2 + 1);
    minmax_fp_bits(a, b, element_size, op)
}

fn minmax_fp_bits(lhs: u128, rhs: u128, element_size: usize, op: Opcode) -> u128 {
    match element_size {
        4 => {
            apply_f32(op, f32::from_bits(lhs as u32), f32::from_bits(rhs as u32)).to_bits() as u128
        }
        8 => {
            apply_f64(op, f64::from_bits(lhs as u64), f64::from_bits(rhs as u64)).to_bits() as u128
        }
        _ => 0,
    }
}

fn pairwise_source(lhs: u128, rhs: u128, element_size: usize, lanes: usize, index: usize) -> u128 {
    if index < lanes {
        simd_element(lhs, index, element_size)
    } else {
        simd_element(rhs, index - lanes, element_size)
    }
}
