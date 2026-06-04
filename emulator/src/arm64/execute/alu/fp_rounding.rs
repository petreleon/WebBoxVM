use super::*;

pub(in crate::arm64::execute) fn exec_fp_rounding(cpu: &mut Armv8Cpu, instr: Instr) {
    match instr.op {
        Opcode::FpFrintm => {
            if instr.size == 4 {
                let value = f32::from_bits(read_fp_bits(cpu, instr.rn, 4) as u32).floor();
                write_fp_bits(cpu, instr.rd, value.to_bits() as u64, 4);
            } else {
                let value = f64::from_bits(read_fp_bits(cpu, instr.rn, 8)).floor();
                write_fp_bits(cpu, instr.rd, value.to_bits(), 8);
            }
        }
        Opcode::FpFrintn => {
            if instr.size == 4 {
                let value = f32::from_bits(read_fp_bits(cpu, instr.rn, 4) as u32) as f64;
                write_fp_bits(
                    cpu,
                    instr.rd,
                    (round_ties_even(value) as f32).to_bits() as u64,
                    4,
                );
            } else {
                let value = f64::from_bits(read_fp_bits(cpu, instr.rn, 8));
                write_fp_bits(cpu, instr.rd, round_ties_even(value).to_bits(), 8);
            }
        }
        Opcode::FpFrinta => {
            if instr.size == 4 {
                let value = f32::from_bits(read_fp_bits(cpu, instr.rn, 4) as u32).round();
                write_fp_bits(cpu, instr.rd, value.to_bits() as u64, 4);
            } else {
                let value = f64::from_bits(read_fp_bits(cpu, instr.rn, 8)).round();
                write_fp_bits(cpu, instr.rd, value.to_bits(), 8);
            }
        }
        Opcode::FpFrintx => {
            if instr.size == 4 {
                let value = f32::from_bits(read_fp_bits(cpu, instr.rn, 4) as u32) as f64;
                write_fp_bits(
                    cpu,
                    instr.rd,
                    (round_fpcr(value, cpu.sys.fpcr) as f32).to_bits() as u64,
                    4,
                );
            } else {
                let value = f64::from_bits(read_fp_bits(cpu, instr.rn, 8));
                write_fp_bits(cpu, instr.rd, round_fpcr(value, cpu.sys.fpcr).to_bits(), 8);
            }
        }
        Opcode::FpFrintp => {
            if instr.size == 4 {
                let value = f32::from_bits(read_fp_bits(cpu, instr.rn, 4) as u32).ceil();
                write_fp_bits(cpu, instr.rd, value.to_bits() as u64, 4);
            } else {
                let value = f64::from_bits(read_fp_bits(cpu, instr.rn, 8)).ceil();
                write_fp_bits(cpu, instr.rd, value.to_bits(), 8);
            }
        }
        Opcode::FpFrintz => {
            if instr.size == 4 {
                let value = f32::from_bits(read_fp_bits(cpu, instr.rn, 4) as u32).trunc();
                write_fp_bits(cpu, instr.rd, value.to_bits() as u64, 4);
            } else {
                let value = f64::from_bits(read_fp_bits(cpu, instr.rn, 8)).trunc();
                write_fp_bits(cpu, instr.rd, value.to_bits(), 8);
            }
        }
        Opcode::FpFrinti => {
            if instr.size == 4 {
                let value = f32::from_bits(read_fp_bits(cpu, instr.rn, 4) as u32) as f64;
                write_fp_bits(
                    cpu,
                    instr.rd,
                    (round_fpcr(value, cpu.sys.fpcr) as f32).to_bits() as u64,
                    4,
                );
            } else {
                let value = f64::from_bits(read_fp_bits(cpu, instr.rn, 8));
                write_fp_bits(cpu, instr.rd, round_fpcr(value, cpu.sys.fpcr).to_bits(), 8);
            }
        }
        _ => unreachable!(),
    }
}
