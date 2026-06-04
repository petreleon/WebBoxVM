use super::*;

pub(in crate::arm64::execute) fn exec_simd_fp_compare(cpu: &mut Armv8Cpu, instr: Instr) {
    let element_size = instr.imm.max(1) as usize;
    let vector_size = instr.size as usize;
    let lanes = vector_size / element_size;
    let element_mask = simd_element_mask(element_size);
    let mut out = 0u128;

    for lane in 0..lanes {
        if compare_passes(cpu, instr, lane, element_size) {
            out |= element_mask << (lane * element_size * 8);
        }
    }

    cpu.simd[instr.rd as usize] = out & simd_vector_mask(vector_size);
}

fn compare_passes(cpu: &Armv8Cpu, instr: Instr, lane: usize, element_size: usize) -> bool {
    let lhs = simd_element(cpu.simd[instr.rn as usize], lane, element_size);
    let rhs = compare_rhs(cpu, instr, lane, element_size);
    match element_size {
        4 => {
            let left = f32::from_bits(lhs as u32);
            compare_fp(instr.op, left, f32::from_bits(rhs as u32))
        }
        8 => {
            let left = f64::from_bits(lhs as u64);
            compare_fp(instr.op, left, f64::from_bits(rhs as u64))
        }
        _ => false,
    }
}

fn compare_rhs(cpu: &Armv8Cpu, instr: Instr, lane: usize, element_size: usize) -> u128 {
    if matches!(
        instr.op,
        Opcode::SimdFpFcmeqZero
            | Opcode::SimdFpFcmgeZero
            | Opcode::SimdFpFcmgtZero
            | Opcode::SimdFpFcmleZero
            | Opcode::SimdFpFcmltZero
    ) {
        0
    } else {
        simd_element(cpu.simd[instr.rm as usize], lane, element_size)
    }
}

fn compare_fp<T>(op: Opcode, left: T, right: T) -> bool
where
    T: PartialOrd + Copy + AbsValue,
{
    match op {
        Opcode::SimdFpFacgeVec => left.abs_value() >= right.abs_value(),
        Opcode::SimdFpFacgtVec => left.abs_value() > right.abs_value(),
        Opcode::SimdFpFcmgeVec => left >= right,
        Opcode::SimdFpFcmgtVec => left > right,
        Opcode::SimdFpFcmeqZero => left == right,
        Opcode::SimdFpFcmgeZero => left >= right,
        Opcode::SimdFpFcmgtZero => left > right,
        Opcode::SimdFpFcmleZero => left <= right,
        Opcode::SimdFpFcmltZero => left < right,
        _ => unreachable!(),
    }
}

trait AbsValue {
    fn abs_value(self) -> Self;
}

impl AbsValue for f32 {
    fn abs_value(self) -> Self {
        self.abs()
    }
}

impl AbsValue for f64 {
    fn abs_value(self) -> Self {
        self.abs()
    }
}
