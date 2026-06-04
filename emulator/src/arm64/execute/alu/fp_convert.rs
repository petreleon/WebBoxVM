use super::*;

pub(in crate::arm64::execute) fn exec_fp_convert(cpu: &mut Armv8Cpu, instr: Instr) {
    match instr.op {
        Opcode::Scvtf => {
            let value = if instr.sf {
                read_reg(cpu, instr.rn, true) as i64 as f64
            } else {
                read_reg(cpu, instr.rn, false) as u32 as i32 as f64
            };
            let scaled = if instr.cond == 1 {
                value / 2f64.powi(instr.imm as i32)
            } else {
                value
            };
            if instr.size == 4 {
                write_fp_bits(cpu, instr.rd, (scaled as f32).to_bits() as u64, 4);
            } else {
                write_fp_bits(cpu, instr.rd, scaled.to_bits(), 8);
            }
        }
        Opcode::Ucvtf => {
            let value = if instr.sf {
                read_reg(cpu, instr.rn, true) as f64
            } else {
                read_reg(cpu, instr.rn, false) as u32 as f64
            };
            let scaled = if instr.cond == 1 {
                value / 2f64.powi(instr.imm as i32)
            } else {
                value
            };
            if instr.size == 4 {
                write_fp_bits(cpu, instr.rd, (scaled as f32).to_bits() as u64, 4);
            } else {
                write_fp_bits(cpu, instr.rd, scaled.to_bits(), 8);
            }
        }
        Opcode::Fcvtns => {
            let value = round_ties_even(read_fp_as_f64(cpu, instr.rn, instr.size));
            if instr.sf {
                write_reg(cpu, instr.rd, value as i64 as u64, true);
            } else {
                write_reg(cpu, instr.rd, value as i32 as u32 as u64, false);
            }
        }
        Opcode::Fcvtms => {
            let value = read_fp_as_f64(cpu, instr.rn, instr.size).floor();
            if instr.sf {
                write_reg(cpu, instr.rd, value as i64 as u64, true);
            } else {
                write_reg(cpu, instr.rd, value as i32 as u32 as u64, false);
            }
        }
        Opcode::Fcvtzs => {
            let mut value = read_fp_as_f64(cpu, instr.rn, instr.size);
            if instr.cond == 1 {
                value *= 2f64.powi(instr.imm as i32);
            }
            let value = value.trunc();
            if instr.sf {
                write_reg(cpu, instr.rd, value as i64 as u64, true);
            } else {
                write_reg(cpu, instr.rd, value as i32 as u32 as u64, false);
            }
        }
        Opcode::Fcvtzu => {
            let mut value = read_fp_as_f64(cpu, instr.rn, instr.size);
            if instr.cond == 1 {
                value *= 2f64.powi(instr.imm as i32);
            }
            let value = value.trunc();
            if instr.sf {
                write_reg(cpu, instr.rd, value as u64, true);
            } else {
                write_reg(cpu, instr.rd, value as u32 as u64, false);
            }
        }
        Opcode::Fcvtas => {
            let value = read_fp_as_f64(cpu, instr.rn, instr.size).round();
            if instr.sf {
                write_reg(cpu, instr.rd, value as i64 as u64, true);
            } else {
                write_reg(cpu, instr.rd, value as i32 as u32 as u64, false);
            }
        }
        _ => unreachable!(),
    }
}
