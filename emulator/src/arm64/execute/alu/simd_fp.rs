use super::*;

pub(in crate::arm64::execute) fn exec_simd_fp(cpu: &mut Armv8Cpu, instr: Instr) {
    let rd = instr.rd as usize;
    let rn = instr.rn as usize;
    let rm = instr.rm as usize;

    match instr.op {
        Opcode::SimdScvtf => {
            let element_size = instr.imm.max(1) as usize;
            let lanes = (instr.size as usize / element_size).max(1);
            let mut out = 0u128;
            match element_size {
                4 => {
                    for lane in 0..lanes {
                        let value = simd_element(cpu.simd[rn], lane, element_size) as u32 as i32;
                        out |= ((value as f32).to_bits() as u128) << (lane * 32);
                    }
                }
                8 => {
                    for lane in 0..lanes {
                        let value = simd_element(cpu.simd[rn], lane, element_size) as u64 as i64;
                        out |= ((value as f64).to_bits() as u128) << (lane * 64);
                    }
                }
                _ => {}
            }
            cpu.simd[rd] = out & simd_vector_mask(instr.size as usize);
        }
        Opcode::SimdFcvtzs => {
            let element_size = instr.imm.max(1) as usize;
            let lanes = (instr.size as usize / element_size).max(1);
            let mut out = 0u128;
            match element_size {
                4 => {
                    for lane in 0..lanes {
                        let value =
                            f32::from_bits(simd_element(cpu.simd[rn], lane, element_size) as u32)
                                .trunc() as i32 as u32;
                        out |= (value as u128) << (lane * 32);
                    }
                }
                8 => {
                    for lane in 0..lanes {
                        let value =
                            f64::from_bits(simd_element(cpu.simd[rn], lane, element_size) as u64)
                                .trunc() as i64 as u64;
                        out |= (value as u128) << (lane * 64);
                    }
                }
                _ => {}
            }
            cpu.simd[rd] = out & simd_vector_mask(instr.size as usize);
        }
        Opcode::SimdFcvtzu => {
            let value = if instr.size == 4 {
                f32::from_bits(read_fp_bits(cpu, instr.rn, 4) as u32).trunc() as u32 as u64
            } else {
                f64::from_bits(read_fp_bits(cpu, instr.rn, 8)).trunc() as u64
            };
            write_fp_bits(cpu, instr.rd, value, instr.size);
        }
        Opcode::SimdFpAddVec => {
            cpu.simd[rd] = simd_fp_elementwise_binary(
                cpu.simd[rn],
                cpu.simd[rm],
                instr.imm.max(1) as usize,
                instr.size as usize,
                |a, b| a + b,
                |a, b| a + b,
            );
        }
        Opcode::SimdFpSubVec => {
            cpu.simd[rd] = simd_fp_elementwise_binary(
                cpu.simd[rn],
                cpu.simd[rm],
                instr.imm.max(1) as usize,
                instr.size as usize,
                |a, b| a - b,
                |a, b| a - b,
            );
        }
        Opcode::SimdFpMulVec => {
            cpu.simd[rd] = simd_fp_elementwise_binary(
                cpu.simd[rn],
                cpu.simd[rm],
                instr.imm.max(1) as usize,
                instr.size as usize,
                |a, b| a * b,
                |a, b| a * b,
            );
        }
        Opcode::SimdFpDivVec => {
            cpu.simd[rd] = simd_fp_elementwise_binary(
                cpu.simd[rn],
                cpu.simd[rm],
                instr.imm.max(1) as usize,
                instr.size as usize,
                |a, b| a / b,
                |a, b| a / b,
            );
        }
        Opcode::SimdFpAbd => {
            cpu.simd[rd] = simd_fp_elementwise_binary(
                cpu.simd[rn],
                cpu.simd[rm],
                instr.imm.max(1) as usize,
                instr.size as usize,
                |a, b| (a - b).abs(),
                |a, b| (a - b).abs(),
            );
        }
        Opcode::SimdFpNeg => {
            let element_size = instr.imm.max(1) as usize;
            let bits = element_size * 8;
            let sign_bit = 1u128 << (bits - 1);
            let lanes = instr.size as usize / element_size;
            let mut out = 0u128;
            for lane in 0..lanes {
                let value = simd_element(cpu.simd[rn], lane, element_size) ^ sign_bit;
                out |= value << (lane * bits);
            }
            cpu.simd[rd] = out;
        }
        _ => unreachable!(),
    }
}
