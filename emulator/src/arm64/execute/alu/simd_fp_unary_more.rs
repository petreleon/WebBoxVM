use super::*;

pub(in crate::arm64::execute) fn exec_simd_fp_unary_more(cpu: &mut Armv8Cpu, instr: Instr) {
    let element_size = instr.imm.max(1) as usize;
    let vector_size = instr.size as usize;
    cpu.simd[instr.rd as usize] = match instr.op {
        Opcode::SimdFpAbsVec => simd_fp_abs(cpu.simd[instr.rn as usize], element_size, vector_size),
        Opcode::SimdFpFrintaVec => {
            simd_fp_round(cpu.simd[instr.rn as usize], element_size, vector_size)
        }
        _ => unreachable!(),
    };
}

fn simd_fp_abs(value: u128, element_size: usize, vector_size: usize) -> u128 {
    let bits = element_size * 8;
    let sign_bit = 1u128 << (bits - 1);
    let lanes = vector_size / element_size;
    let mut out = 0u128;
    for lane in 0..lanes {
        let element = simd_element(value, lane, element_size) & !sign_bit;
        out |= element << (lane * bits);
    }
    out & simd_vector_mask(vector_size)
}

fn simd_fp_round(value: u128, element_size: usize, vector_size: usize) -> u128 {
    match element_size {
        4 => simd_fp_elementwise_unary(value, element_size, vector_size, |bits| {
            f32::from_bits(bits as u32).round().to_bits() as u64
        }),
        8 => simd_fp_elementwise_unary(value, element_size, vector_size, |bits| {
            f64::from_bits(bits).round().to_bits()
        }),
        _ => 0,
    }
}

fn simd_fp_elementwise_unary<F>(value: u128, element_size: usize, vector_size: usize, op: F) -> u128
where
    F: Fn(u64) -> u64,
{
    let bits = element_size * 8;
    let lanes = vector_size / element_size;
    let mask = simd_element_mask(element_size) as u64;
    let mut out = 0u128;
    for lane in 0..lanes {
        let element = simd_element(value, lane, element_size) as u64;
        out |= ((op(element) & mask) as u128) << (lane * bits);
    }
    out & simd_vector_mask(vector_size)
}
