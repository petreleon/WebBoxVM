use super::*;

pub(in crate::arm64::execute) fn exec_simd_fp_compare(cpu: &mut Armv8Cpu, instr: Instr) {
    let element_size = instr.imm.max(1) as usize;
    let vector_size = instr.size as usize;
    let lanes = vector_size / element_size;
    let element_mask = simd_element_mask(element_size);
    let mut out = 0u128;

    for lane in 0..lanes {
        if abs_compare_passes(cpu, instr, lane, element_size) {
            out |= element_mask << (lane * element_size * 8);
        }
    }

    cpu.simd[instr.rd as usize] = out & simd_vector_mask(vector_size);
}

fn abs_compare_passes(cpu: &Armv8Cpu, instr: Instr, lane: usize, element_size: usize) -> bool {
    let lhs = simd_element(cpu.simd[instr.rn as usize], lane, element_size);
    let rhs = simd_element(cpu.simd[instr.rm as usize], lane, element_size);
    match element_size {
        4 => {
            let left = f32::from_bits(lhs as u32).abs();
            let right = f32::from_bits(rhs as u32).abs();
            compare_abs(instr.op, left, right)
        }
        8 => {
            let left = f64::from_bits(lhs as u64).abs();
            let right = f64::from_bits(rhs as u64).abs();
            compare_abs(instr.op, left, right)
        }
        _ => false,
    }
}

fn compare_abs<T: PartialOrd>(op: Opcode, left: T, right: T) -> bool {
    match op {
        Opcode::SimdFpFacgeVec => left >= right,
        Opcode::SimdFpFacgtVec => left > right,
        _ => unreachable!(),
    }
}
