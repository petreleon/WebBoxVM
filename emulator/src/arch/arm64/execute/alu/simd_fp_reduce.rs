use super::*;

pub(in crate::arch::arm64::execute) fn exec_simd_fp_reduce(cpu: &mut Armv8Cpu, instr: Instr) {
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
