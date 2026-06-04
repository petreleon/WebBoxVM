use super::*;

pub(in crate::arm64::execute) fn exec_simd_fp_reduce(cpu: &mut Armv8Cpu, instr: Instr) {
    let result = reduce_f32x4(cpu.simd[instr.rn as usize], instr.op);
    write_fp_bits(cpu, instr.rd, result.to_bits() as u64, 4);
}

fn reduce_f32x4(value: u128, op: Opcode) -> f32 {
    let mut result = f32::from_bits(simd_element(value, 0, 4) as u32);
    for lane in 1..4 {
        let element = f32::from_bits(simd_element(value, lane, 4) as u32);
        result = match op {
            Opcode::SimdFpFmaxv => fp_max(result, element),
            Opcode::SimdFpFminv => fp_min(result, element),
            Opcode::SimdFpFmaxnmv => fp_max_num(result, element),
            Opcode::SimdFpFminnmv => fp_min_num(result, element),
            _ => unreachable!(),
        };
    }
    result
}

fn fp_max(lhs: f32, rhs: f32) -> f32 {
    if lhs.is_nan() || rhs.is_nan() {
        f32::NAN
    } else if both_zero(lhs, rhs) {
        if lhs.is_sign_positive() || rhs.is_sign_positive() {
            0.0
        } else {
            -0.0
        }
    } else {
        lhs.max(rhs)
    }
}

fn fp_min(lhs: f32, rhs: f32) -> f32 {
    if lhs.is_nan() || rhs.is_nan() {
        f32::NAN
    } else if both_zero(lhs, rhs) {
        if lhs.is_sign_negative() || rhs.is_sign_negative() {
            -0.0
        } else {
            0.0
        }
    } else {
        lhs.min(rhs)
    }
}

fn fp_max_num(lhs: f32, rhs: f32) -> f32 {
    match (lhs.is_nan(), rhs.is_nan()) {
        (true, false) => rhs,
        (false, true) => lhs,
        _ => fp_max(lhs, rhs),
    }
}

fn fp_min_num(lhs: f32, rhs: f32) -> f32 {
    match (lhs.is_nan(), rhs.is_nan()) {
        (true, false) => rhs,
        (false, true) => lhs,
        _ => fp_min(lhs, rhs),
    }
}

fn both_zero(lhs: f32, rhs: f32) -> bool {
    lhs == 0.0 && rhs == 0.0
}
