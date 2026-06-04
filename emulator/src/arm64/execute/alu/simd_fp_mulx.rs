use super::*;

pub(in crate::arm64::execute) fn exec_simd_fp_mulx(cpu: &mut Armv8Cpu, instr: Instr) {
    if instr.size == instr.imm as u8 {
        exec_scalar(cpu, instr);
        return;
    }
    cpu.simd[instr.rd as usize] = simd_fp_elementwise_binary(
        cpu.simd[instr.rn as usize],
        cpu.simd[instr.rm as usize],
        instr.imm.max(1) as usize,
        instr.size as usize,
        fmulx32,
        fmulx64,
    );
}

pub(in crate::arm64::execute) fn exec_simd_fp_mulx_elem(cpu: &mut Armv8Cpu, instr: Instr) {
    let element_size = instr.imm.max(1) as usize;
    let vector_size = instr.size as usize;
    let lane_value = simd_element(
        cpu.simd[instr.rm as usize],
        instr.cond as usize,
        element_size,
    );
    let rhs = broadcast_lane(lane_value, element_size, vector_size);
    cpu.simd[instr.rd as usize] = simd_fp_elementwise_binary(
        cpu.simd[instr.rn as usize],
        rhs,
        element_size,
        vector_size,
        fmulx32,
        fmulx64,
    );
}

fn exec_scalar(cpu: &mut Armv8Cpu, instr: Instr) {
    match instr.size {
        4 => {
            let lhs = f32::from_bits(read_fp_bits(cpu, instr.rn, 4) as u32);
            let rhs = f32::from_bits(read_fp_bits(cpu, instr.rm, 4) as u32);
            write_fp_bits(cpu, instr.rd, fmulx32(lhs, rhs).to_bits() as u64, 4);
        }
        8 => {
            let lhs = f64::from_bits(read_fp_bits(cpu, instr.rn, 8));
            let rhs = f64::from_bits(read_fp_bits(cpu, instr.rm, 8));
            write_fp_bits(cpu, instr.rd, fmulx64(lhs, rhs).to_bits(), 8);
        }
        _ => {}
    }
}

fn broadcast_lane(lane_value: u128, element_size: usize, vector_size: usize) -> u128 {
    let mut out = 0u128;
    for lane in 0..vector_size / element_size {
        out |= lane_value << (lane * element_size * 8);
    }
    out
}

fn fmulx32(lhs: f32, rhs: f32) -> f32 {
    if (lhs == 0.0 && rhs.is_infinite()) || (rhs == 0.0 && lhs.is_infinite()) {
        signed_two(lhs.is_sign_negative() ^ rhs.is_sign_negative()) as f32
    } else {
        lhs * rhs
    }
}

fn fmulx64(lhs: f64, rhs: f64) -> f64 {
    if (lhs == 0.0 && rhs.is_infinite()) || (rhs == 0.0 && lhs.is_infinite()) {
        signed_two(lhs.is_sign_negative() ^ rhs.is_sign_negative())
    } else {
        lhs * rhs
    }
}

fn signed_two(negative: bool) -> f64 {
    if negative { -2.0 } else { 2.0 }
}
