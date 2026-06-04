use super::*;

pub(in crate::arm64::execute) fn exec_simd_fp_convert(cpu: &mut Armv8Cpu, instr: Instr) {
    match instr.op {
        Opcode::SimdFcvtas => exec_simd_fcvtas(cpu, instr),
        _ => unreachable!(),
    }
}

fn exec_simd_fcvtas(cpu: &mut Armv8Cpu, instr: Instr) {
    let rd = instr.rd as usize;
    let rn = instr.rn as usize;
    let element_size = instr.imm.max(1) as usize;
    let vector_size = instr.size as usize;
    let lanes = vector_size / element_size;
    let mut out = 0u128;
    for lane in 0..lanes {
        let value = simd_element(cpu.simd[rn], lane, element_size);
        let converted = match element_size {
            4 => f32::from_bits(value as u32).round() as i32 as u32 as u64,
            8 => f64::from_bits(value as u64).round() as i64 as u64,
            _ => 0,
        };
        out |= (converted as u128) << (lane * element_size * 8);
    }
    cpu.simd[rd] = out & simd_vector_mask(vector_size);
}
