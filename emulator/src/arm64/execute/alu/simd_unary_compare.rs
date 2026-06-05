use super::*;

pub(in crate::arm64::execute) fn exec_simd_unary_compare(cpu: &mut Armv8Cpu, instr: Instr) {
    let rd = instr.rd as usize;
    let rn = instr.rn as usize;
    let rm = instr.rm as usize;

    match instr.op {
        Opcode::SimdCmeqZero
        | Opcode::SimdCmgtZero
        | Opcode::SimdCmgeZero
        | Opcode::SimdCmleZero
        | Opcode::SimdCmltZero => cpu.simd[rd] = simd_compare_zero(cpu, instr),
        Opcode::SimdCmeqReg => {
            cpu.simd[rd] = simd_compare_register(cpu, instr);
        }
        Opcode::SimdCmgtReg | Opcode::SimdCmgeReg | Opcode::SimdCmhsReg | Opcode::SimdCmhiReg => {
            cpu.simd[rd] = simd_compare_register(cpu, instr);
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
        Opcode::SimdCnt | Opcode::SimdCls | Opcode::SimdClz => {
            cpu.simd[rd] = simd_count(cpu.simd[rn], instr);
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

fn simd_compare_zero(cpu: &Armv8Cpu, instr: Instr) -> u128 {
    let element_size = instr.imm.max(1) as usize;
    let bits = element_size * 8;
    let lanes = (instr.size as usize / element_size).max(1);
    let element_mask = simd_element_mask(element_size);
    let mut out = 0u128;
    for lane in 0..lanes {
        let value = simd_signed_element(cpu.simd[instr.rn as usize], lane, element_size);
        let pass = match instr.op {
            Opcode::SimdCmeqZero => value == 0,
            Opcode::SimdCmgtZero => value > 0,
            Opcode::SimdCmgeZero => value >= 0,
            Opcode::SimdCmleZero => value <= 0,
            Opcode::SimdCmltZero => value < 0,
            _ => false,
        };
        if pass {
            out |= element_mask << (lane * bits);
        }
    }
    out & simd_vector_mask(instr.size as usize)
}

fn simd_count(value: u128, instr: Instr) -> u128 {
    if instr.op == Opcode::SimdCnt {
        let mut out = 0u128;
        for lane in 0..instr.size as usize {
            out |= (simd_byte(value, lane).count_ones() as u128) << (lane * 8);
        }
        return out;
    }
    let element_size = instr.imm.max(1) as usize;
    let bits = element_size * 8;
    let mut out = 0u128;
    for lane in 0..instr.size as usize / element_size {
        let element = simd_element(value, lane, element_size);
        let count = if instr.op == Opcode::SimdClz {
            leading_zero_bits(element, bits)
        } else {
            leading_sign_bits(element, bits)
        };
        out |= (count as u128) << (lane * bits);
    }
    out & simd_vector_mask(instr.size as usize)
}

fn leading_zero_bits(value: u128, bits: usize) -> u32 {
    (value << (128 - bits)).leading_zeros().min(bits as u32)
}

fn leading_sign_bits(value: u128, bits: usize) -> u32 {
    let sign_bit = 1u128 << (bits - 1);
    let count = if (value & sign_bit) == 0 {
        leading_zero_bits(value, bits)
    } else {
        leading_zero_bits(!value & simd_element_mask(bits / 8), bits)
    };
    count - 1
}

fn simd_compare_register(cpu: &Armv8Cpu, instr: Instr) -> u128 {
    let element_size = instr.imm.max(1) as usize;
    simd_elementwise_binary(
        cpu.simd[instr.rn as usize],
        cpu.simd[instr.rm as usize],
        element_size,
        instr.size as usize,
        |a, b, mask| {
            let pass = match instr.op {
                Opcode::SimdCmeqReg => a == b,
                Opcode::SimdCmhiReg => a > b,
                Opcode::SimdCmhsReg => a >= b,
                Opcode::SimdCmgtReg => {
                    simd_signed_element_value(a, element_size)
                        > simd_signed_element_value(b, element_size)
                }
                Opcode::SimdCmgeReg => {
                    simd_signed_element_value(a, element_size)
                        >= simd_signed_element_value(b, element_size)
                }
                _ => false,
            };
            if pass {
                mask
            } else {
                0
            }
        },
    )
}
