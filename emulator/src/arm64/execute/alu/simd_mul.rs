use super::*;

pub(in crate::arm64::execute) fn exec_simd_mul(cpu: &mut Armv8Cpu, instr: Instr) {
    let rd = instr.rd as usize;
    let rn = instr.rn as usize;
    let rm = instr.rm as usize;
    let element_size = instr.imm.max(1) as usize;

    match instr.op {
        Opcode::SimdMulVec => {
            cpu.simd[rd] = simd_elementwise_binary(
                cpu.simd[rn],
                cpu.simd[rm],
                element_size,
                instr.size as usize,
                |a, b, mask| a.wrapping_mul(b) & mask,
            );
        }
        Opcode::SimdMlaVec | Opcode::SimdMlsVec => {
            let element_mask = simd_element_mask(element_size);
            let subtract = instr.op == Opcode::SimdMlsVec;
            cpu.simd[rd] = simd_elementwise_ternary(
                cpu.simd[rd],
                cpu.simd[rn],
                cpu.simd[rm],
                element_size,
                instr.size as usize,
                |acc, a, b| {
                    let product = a.wrapping_mul(b) & element_mask;
                    if subtract {
                        acc.wrapping_sub(product)
                    } else {
                        acc.wrapping_add(product)
                    }
                },
            );
        }
        _ => unreachable!(),
    }
}
