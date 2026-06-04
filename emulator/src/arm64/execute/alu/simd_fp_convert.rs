use super::*;

pub(in crate::arm64::execute) fn exec_simd_fp_convert(cpu: &mut Armv8Cpu, instr: Instr) {
    match instr.op {
        Opcode::SimdFcvtas => exec_simd_fcvtas(cpu, instr),
        Opcode::SimdFcvtl | Opcode::SimdFcvtl2 => exec_simd_fcvtl(cpu, instr),
        Opcode::SimdFcvtn | Opcode::SimdFcvtn2 => exec_simd_fcvtn(cpu, instr),
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

fn exec_simd_fcvtl(cpu: &mut Armv8Cpu, instr: Instr) {
    let rd = instr.rd as usize;
    let rn = instr.rn as usize;
    let src_size = instr.imm as usize;
    let dst_size = instr.cond as usize;
    let source_lane_base = if instr.op == Opcode::SimdFcvtl2 {
        8 / src_size
    } else {
        0
    };
    let mut out = 0u128;
    for lane in 0..(8 / src_size) {
        let value = simd_element(cpu.simd[rn], source_lane_base + lane, src_size);
        let converted = simd_fp_convert_width(value as u64, src_size, dst_size);
        out |= (converted as u128) << (lane * dst_size * 8);
    }
    cpu.simd[rd] = out;
}

fn exec_simd_fcvtn(cpu: &mut Armv8Cpu, instr: Instr) {
    let rd = instr.rd as usize;
    let rn = instr.rn as usize;
    let src_size = instr.imm as usize;
    let dst_size = instr.cond as usize;
    let mut part = 0u128;
    for lane in 0..(8 / dst_size) {
        let value = simd_element(cpu.simd[rn], lane, src_size);
        let converted = simd_fp_convert_width(value as u64, src_size, dst_size);
        part |= (converted as u128) << (lane * dst_size * 8);
    }
    cpu.simd[rd] = if instr.op == Opcode::SimdFcvtn2 {
        (cpu.simd[rd] & u64::MAX as u128) | (part << 64)
    } else {
        part
    };
}

fn simd_fp_convert_width(value: u64, src_size: usize, dst_size: usize) -> u64 {
    match (src_size, dst_size) {
        (2, 4) => f16_to_f32(value as u16).to_bits() as u64,
        (4, 8) => (f32::from_bits(value as u32) as f64).to_bits(),
        (4, 2) => f32_to_f16_bits(f32::from_bits(value as u32)) as u64,
        (8, 4) => (f64::from_bits(value) as f32).to_bits() as u64,
        _ => unreachable!(),
    }
}
