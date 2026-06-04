use super::*;

pub(in crate::arm64::execute) fn exec_simd_unary_compare(cpu: &mut Armv8Cpu, instr: Instr) {
    let rd = instr.rd as usize;
    let rn = instr.rn as usize;
    let rm = instr.rm as usize;

    match instr.op {
        Opcode::SimdCmeqZero => {
            let src = cpu.simd[rn];
            let element_size = instr.imm.max(1) as usize;
            cpu.simd[rd] = simd_compare_elements_with_zero(src, element_size, instr.size as usize);
        }
        Opcode::SimdCmgeZero => {
            let src = cpu.simd[rn];
            let element_size = instr.imm.max(1) as usize;
            let bits = element_size * 8;
            let lanes = (instr.size as usize / element_size).max(1);
            let element_mask = simd_element_mask(element_size);
            let mut out = 0u128;
            for lane in 0..lanes {
                if simd_signed_element(src, lane, element_size) >= 0 {
                    out |= element_mask << (lane * bits);
                }
            }
            cpu.simd[rd] = out & simd_vector_mask(instr.size as usize);
        }
        Opcode::SimdCmeqReg => {
            let lhs = cpu.simd[rn];
            let rhs = cpu.simd[rm];
            let element_size = instr.imm.max(1) as usize;
            cpu.simd[rd] = simd_elementwise_binary(
                lhs,
                rhs,
                element_size,
                instr.size as usize,
                |a, b, mask| if a == b { mask } else { 0 },
            );
        }
        Opcode::SimdCmhsReg => {
            let lhs = cpu.simd[rn];
            let rhs = cpu.simd[rm];
            let element_size = instr.imm.max(1) as usize;
            cpu.simd[rd] = simd_elementwise_binary(
                lhs,
                rhs,
                element_size,
                instr.size as usize,
                |a, b, mask| if a >= b { mask } else { 0 },
            );
        }
        Opcode::SimdCmhiReg => {
            let lhs = cpu.simd[rn];
            let rhs = cpu.simd[rm];
            let element_size = instr.imm.max(1) as usize;
            cpu.simd[rd] = simd_elementwise_binary(
                lhs,
                rhs,
                element_size,
                instr.size as usize,
                |a, b, mask| if a > b { mask } else { 0 },
            );
        }
        Opcode::SimdUqsub => {
            const FPSR_QC: u64 = 1 << 27;

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
        Opcode::SimdAbs => {
            let element_size = instr.imm.max(1) as usize;
            let bits = element_size * 8;
            let lanes = (instr.size as usize / element_size).max(1);
            let element_mask = simd_element_mask(element_size);
            let sign_bit = 1u128 << (bits - 1);
            let mut out = 0u128;
            for lane in 0..lanes {
                let value = simd_element(cpu.simd[rn], lane, element_size);
                let abs = if (value & sign_bit) != 0 {
                    0u128.wrapping_sub(value) & element_mask
                } else {
                    value
                };
                out |= abs << (lane * bits);
            }
            cpu.simd[rd] = out & simd_vector_mask(instr.size as usize);
        }
        Opcode::SimdNeg => {
            let element_size = instr.imm.max(1) as usize;
            let bits = element_size * 8;
            let lanes = (instr.size as usize / element_size).max(1);
            let element_mask = simd_element_mask(element_size);
            let mut out = 0u128;
            for lane in 0..lanes {
                let value = simd_element(cpu.simd[rn], lane, element_size);
                out |= (0u128.wrapping_sub(value) & element_mask) << (lane * bits);
            }
            cpu.simd[rd] = out & simd_vector_mask(instr.size as usize);
        }
        Opcode::SimdCnt => {
            let vector_size = instr.size as usize;
            let mut out = 0u128;
            for lane in 0..vector_size {
                out |= (simd_byte(cpu.simd[rn], lane).count_ones() as u128) << (lane * 8);
            }
            cpu.simd[rd] = out;
        }
        Opcode::SimdCmtst => {
            cpu.simd[rd] = simd_elementwise_binary(
                cpu.simd[rn],
                cpu.simd[rm],
                instr.imm.max(1) as usize,
                instr.size as usize,
                |a, b, mask| if (a & b) != 0 { mask } else { 0 },
            );
        }
        _ => unreachable!(),
    }
}
