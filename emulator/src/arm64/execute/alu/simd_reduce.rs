use super::*;

pub(in crate::arm64::execute) fn exec_simd_reduce(cpu: &mut Armv8Cpu, instr: Instr) {
    let rd = instr.rd as usize;
    let rn = instr.rn as usize;
    let rm = instr.rm as usize;

    match instr.op {
        Opcode::SimdAddp => {
            let element_size = instr.imm.max(1) as usize;
            if instr.rm == 0xFF {
                let element_mask = simd_element_mask(element_size);
                let lhs = simd_element(cpu.simd[rn], 0, element_size);
                let rhs = simd_element(cpu.simd[rn], 1, element_size);
                cpu.simd[rd] = lhs.wrapping_add(rhs) & element_mask;
            } else {
                let lhs = cpu.simd[rn];
                let rhs = cpu.simd[rm];
                cpu.simd[rd] = simd_pairwise_binary(
                    lhs,
                    rhs,
                    element_size,
                    instr.size as usize,
                    |a, b, mask| a.wrapping_add(b) & mask,
                );
            }
        }
        Opcode::SimdAddv => {
            let element_size = instr.imm.max(1) as usize;
            let bits = element_size * 8;
            let element_mask = if bits == 128 {
                u128::MAX
            } else {
                (1u128 << bits) - 1
            };
            let lanes = instr.size as usize / element_size;
            let mut sum = 0u128;
            for lane in 0..lanes {
                sum =
                    sum.wrapping_add(simd_element(cpu.simd[rn], lane, element_size)) & element_mask;
            }
            cpu.simd[rd] = sum;
        }
        Opcode::SimdSmaxv => {
            let element_size = instr.imm.max(1) as usize;
            cpu.simd[rd] = signed_reduce(cpu.simd[rn], element_size, instr.size as usize, true);
        }
        Opcode::SimdSminv => {
            let element_size = instr.imm.max(1) as usize;
            cpu.simd[rd] = signed_reduce(cpu.simd[rn], element_size, instr.size as usize, false);
        }
        Opcode::SimdUmaxv => {
            let element_size = instr.imm.max(1) as usize;
            cpu.simd[rd] = unsigned_reduce(cpu.simd[rn], element_size, instr.size as usize, true);
        }
        Opcode::SimdUminv => {
            let element_size = instr.imm.max(1) as usize;
            cpu.simd[rd] = unsigned_reduce(cpu.simd[rn], element_size, instr.size as usize, false);
        }
        Opcode::SimdSmaxVec => {
            let lhs = cpu.simd[rn];
            let rhs = cpu.simd[rm];
            let element_size = instr.imm.max(1) as usize;
            cpu.simd[rd] =
                simd_elementwise_binary(lhs, rhs, element_size, instr.size as usize, |a, b, _| {
                    signed_max(a, b, element_size)
                });
        }
        Opcode::SimdSminVec => {
            let lhs = cpu.simd[rn];
            let rhs = cpu.simd[rm];
            let element_size = instr.imm.max(1) as usize;
            cpu.simd[rd] =
                simd_elementwise_binary(lhs, rhs, element_size, instr.size as usize, |a, b, _| {
                    signed_min(a, b, element_size)
                });
        }
        Opcode::SimdUmaxVec => {
            let lhs = cpu.simd[rn];
            let rhs = cpu.simd[rm];
            let element_size = instr.imm.max(1) as usize;
            cpu.simd[rd] =
                simd_elementwise_binary(lhs, rhs, element_size, instr.size as usize, |a, b, _| {
                    a.max(b)
                });
        }
        Opcode::SimdUminVec => {
            let lhs = cpu.simd[rn];
            let rhs = cpu.simd[rm];
            let element_size = instr.imm.max(1) as usize;
            cpu.simd[rd] =
                simd_elementwise_binary(lhs, rhs, element_size, instr.size as usize, |a, b, _| {
                    a.min(b)
                });
        }
        Opcode::SimdUmaxp => {
            let lhs = cpu.simd[rn];
            let rhs = cpu.simd[rm];
            let element_size = instr.imm.max(1) as usize;
            cpu.simd[rd] =
                simd_pairwise_binary(lhs, rhs, element_size, instr.size as usize, |a, b, _| {
                    a.max(b)
                });
        }
        Opcode::SimdSmaxp => {
            let lhs = cpu.simd[rn];
            let rhs = cpu.simd[rm];
            let element_size = instr.imm.max(1) as usize;
            cpu.simd[rd] =
                simd_pairwise_binary(lhs, rhs, element_size, instr.size as usize, |a, b, _| {
                    signed_max(a, b, element_size)
                });
        }
        Opcode::SimdSminp => {
            let lhs = cpu.simd[rn];
            let rhs = cpu.simd[rm];
            let element_size = instr.imm.max(1) as usize;
            cpu.simd[rd] =
                simd_pairwise_binary(lhs, rhs, element_size, instr.size as usize, |a, b, _| {
                    signed_min(a, b, element_size)
                });
        }
        Opcode::SimdUminp => {
            let lhs = cpu.simd[rn];
            let rhs = cpu.simd[rm];
            let element_size = instr.imm.max(1) as usize;
            cpu.simd[rd] =
                simd_pairwise_binary(lhs, rhs, element_size, instr.size as usize, |a, b, _| {
                    a.min(b)
                });
        }
        _ => unreachable!(),
    }
}
