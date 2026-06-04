use super::*;

pub(in crate::arm64::execute) fn exec_simd_fp_minmax(cpu: &mut Armv8Cpu, instr: Instr) {
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
        _ => unreachable!(),
    }
}

fn apply_f64(op: Opcode, lhs: f64, rhs: f64) -> f64 {
    match op {
        Opcode::SimdFpFmaxVec => fp_max(lhs, rhs),
        Opcode::SimdFpFminVec => fp_min(lhs, rhs),
        Opcode::SimdFpFmaxnmVec => fp_max_num(lhs, rhs),
        Opcode::SimdFpFminnmVec => fp_min_num(lhs, rhs),
        _ => unreachable!(),
    }
}
