use super::*;

pub(in crate::arch::arm64::execute) fn exec_simd_fp_pairwise(cpu: &mut Armv8Cpu, instr: Instr) {
    let element_size = instr.imm.max(1) as usize;
    if instr.size == instr.imm as u8 {
        exec_scalar_pairwise_add(cpu, instr, element_size);
        return;
    }
    cpu.simd[instr.rd as usize] = pairwise_add(
        cpu.simd[instr.rn as usize],
        cpu.simd[instr.rm as usize],
        element_size,
        instr.size as usize,
    );
}

fn exec_scalar_pairwise_add(cpu: &mut Armv8Cpu, instr: Instr, element_size: usize) {
    let src = cpu.simd[instr.rn as usize];
    let bits = add_fp_bits(
        simd_element(src, 0, element_size),
        simd_element(src, 1, element_size),
        element_size,
    );
    write_fp_bits(cpu, instr.rd, bits as u64, instr.size);
}

fn pairwise_add(lhs: u128, rhs: u128, element_size: usize, vector_size: usize) -> u128 {
    let lanes = vector_size / element_size;
    let mut out = 0u128;
    for lane in 0..lanes {
        let a = pairwise_source(lhs, rhs, element_size, lanes, lane * 2);
        let b = pairwise_source(lhs, rhs, element_size, lanes, lane * 2 + 1);
        out |= add_fp_bits(a, b, element_size) << (lane * element_size * 8);
    }
    out & simd_vector_mask(vector_size)
}

fn pairwise_source(lhs: u128, rhs: u128, element_size: usize, lanes: usize, index: usize) -> u128 {
    if index < lanes {
        simd_element(lhs, index, element_size)
    } else {
        simd_element(rhs, index - lanes, element_size)
    }
}

fn add_fp_bits(lhs: u128, rhs: u128, element_size: usize) -> u128 {
    match element_size {
        4 => (f32::from_bits(lhs as u32) + f32::from_bits(rhs as u32)).to_bits() as u128,
        8 => (f64::from_bits(lhs as u64) + f64::from_bits(rhs as u64)).to_bits() as u128,
        _ => 0,
    }
}
