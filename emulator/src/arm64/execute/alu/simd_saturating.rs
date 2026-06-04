use super::*;

const FPSR_QC: u64 = 1 << 27;

pub(in crate::arm64::execute) fn exec_simd_saturating(cpu: &mut Armv8Cpu, instr: Instr) {
    match instr.op {
        Opcode::SimdUqsub => exec_scalar_uqsub(cpu, instr),
        Opcode::SimdUqadd => exec_vector_uqadd(cpu, instr),
        _ => unreachable!(),
    }
}

fn exec_scalar_uqsub(cpu: &mut Armv8Cpu, instr: Instr) {
    let rd = instr.rd as usize;
    let rn = instr.rn as usize;
    let rm = instr.rm as usize;
    let element_size = instr.imm.max(1) as usize;
    let mask = simd_element_mask(element_size);
    let lhs = cpu.simd[rn] & mask;
    let rhs = cpu.simd[rm] & mask;
    let (value, saturated) = if lhs < rhs {
        (0, true)
    } else {
        (lhs - rhs, false)
    };
    if saturated {
        cpu.sys.fpsr |= FPSR_QC;
    }
    cpu.simd[rd] = value;
}

fn exec_vector_uqadd(cpu: &mut Armv8Cpu, instr: Instr) {
    let rd = instr.rd as usize;
    let rn = instr.rn as usize;
    let rm = instr.rm as usize;
    let element_size = instr.imm.max(1) as usize;
    let bits = element_size * 8;
    let lanes = (instr.size as usize / element_size).max(1);
    let mask = simd_element_mask(element_size);
    let mut out = 0u128;
    let mut saturated = false;
    for lane in 0..lanes {
        let lhs = simd_element(cpu.simd[rn], lane, element_size);
        let rhs = simd_element(cpu.simd[rm], lane, element_size);
        let sum = lhs + rhs;
        let value = if sum > mask {
            saturated = true;
            mask
        } else {
            sum
        };
        out |= value << (lane * bits);
    }
    if saturated {
        cpu.sys.fpsr |= FPSR_QC;
    }
    cpu.simd[rd] = out & simd_vector_mask(instr.size as usize);
}
