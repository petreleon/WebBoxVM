use super::*;

pub(in crate::arch::arm64::execute) fn exec_fp_unary(cpu: &mut Armv8Cpu, instr: Instr) {
    match instr.op {
        Opcode::FpNeg => {
            let sign_mask = match instr.size {
                2 => 1u64 << 15,
                4 => 1u64 << 31,
                _ => 1u64 << 63,
            };
            write_fp_bits(
                cpu,
                instr.rd,
                read_fp_bits(cpu, instr.rn, instr.size) ^ sign_mask,
                instr.size,
            );
        }
        Opcode::FpAbs => {
            let sign_mask = match instr.size {
                2 => 1u64 << 15,
                4 => 1u64 << 31,
                _ => 1u64 << 63,
            };
            write_fp_bits(
                cpu,
                instr.rd,
                read_fp_bits(cpu, instr.rn, instr.size) & !sign_mask,
                instr.size,
            );
        }
        Opcode::FpSqrt => {
            if instr.size == 2 {
                let value = f16_to_f32(read_fp_bits(cpu, instr.rn, 2) as u16).sqrt();
                write_fp_bits(cpu, instr.rd, f32_to_f16_bits(value) as u64, 2);
            } else if instr.size == 4 {
                let value = f32::from_bits(read_fp_bits(cpu, instr.rn, 4) as u32).sqrt();
                write_fp_bits(cpu, instr.rd, value.to_bits() as u64, 4);
            } else {
                let value = f64::from_bits(read_fp_bits(cpu, instr.rn, 8)).sqrt();
                write_fp_bits(cpu, instr.rd, value.to_bits(), 8);
            }
        }
        Opcode::FpFcvt => {
            let src_size = instr.cond;
            let bits = match (src_size, instr.size) {
                (2, 4) => {
                    let value = f16_to_f32(read_fp_bits(cpu, instr.rn, 2) as u16);
                    value.to_bits() as u64
                }
                (2, 8) => {
                    let value = f16_to_f32(read_fp_bits(cpu, instr.rn, 2) as u16) as f64;
                    value.to_bits()
                }
                (4, 2) => {
                    let value = f32::from_bits(read_fp_bits(cpu, instr.rn, 4) as u32);
                    f32_to_f16_bits(value) as u64
                }
                (4, 8) => {
                    let value = f32::from_bits(read_fp_bits(cpu, instr.rn, 4) as u32) as f64;
                    value.to_bits()
                }
                (8, 2) => {
                    let value = f64::from_bits(read_fp_bits(cpu, instr.rn, 8));
                    f64_to_f16_bits(value) as u64
                }
                (8, 4) => {
                    let value = f64::from_bits(read_fp_bits(cpu, instr.rn, 8)) as f32;
                    value.to_bits() as u64
                }
                _ => unreachable!(),
            };
            write_fp_bits(cpu, instr.rd, bits, instr.size);
        }
        Opcode::FpMovImm => {
            write_fp_bits(
                cpu,
                instr.rd,
                fp_expand_imm(instr.imm as u8, instr.size),
                instr.size,
            );
        }
        _ => unreachable!(),
    }
}
