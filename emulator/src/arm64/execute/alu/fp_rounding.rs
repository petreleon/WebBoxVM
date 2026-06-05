use super::*;

pub(in crate::arm64::execute) fn exec_fp_rounding(cpu: &mut Armv8Cpu, instr: Instr) {
    let value = read_fp_as_f64(cpu, instr.rn, instr.size);
    let rounded = match instr.op {
        Opcode::FpFrintm => value.floor(),
        Opcode::FpFrintn => round_ties_even(value),
        Opcode::FpFrinta => value.round(),
        Opcode::FpFrintx | Opcode::FpFrinti => round_fpcr(value, cpu.sys.fpcr),
        Opcode::FpFrintp => value.ceil(),
        Opcode::FpFrintz => value.trunc(),
        _ => unreachable!(),
    };
    write_fp_from_f64(cpu, instr.rd, rounded, instr.size);
}

fn write_fp_from_f64(cpu: &mut Armv8Cpu, rd: u8, value: f64, size: u8) {
    match size {
        2 => write_fp_bits(cpu, rd, f32_to_f16_bits(value as f32) as u64, 2),
        4 => write_fp_bits(cpu, rd, (value as f32).to_bits() as u64, 4),
        _ => write_fp_bits(cpu, rd, value.to_bits(), 8),
    }
}
