use super::*;

pub(in crate::arm64::execute) fn exec_fp_compare(cpu: &mut Armv8Cpu, instr: Instr) {
    match instr.op {
        Opcode::Fcmp | Opcode::Fcmpe => {
            let lhs = read_fp_as_f64(cpu, instr.rn, instr.size);
            let rhs = if instr.cond == 1 {
                0.0
            } else {
                read_fp_as_f64(cpu, instr.rm, instr.size)
            };
            set_fp_compare_flags(cpu, lhs, rhs);
        }
        Opcode::Fccmp | Opcode::Fccmpe => {
            if cond_taken(cpu, instr.cond) {
                let lhs = read_fp_as_f64(cpu, instr.rn, instr.size);
                let rhs = read_fp_as_f64(cpu, instr.rm, instr.size);
                set_fp_compare_flags(cpu, lhs, rhs);
            } else {
                set_nzcv_from_bits(cpu, instr.imm);
            }
        }
        Opcode::Fcsel => {
            let src = if cond_taken(cpu, instr.cond) {
                instr.rn
            } else {
                instr.rm
            };
            write_fp_bits(
                cpu,
                instr.rd,
                read_fp_bits(cpu, src, instr.size),
                instr.size,
            );
        }
        _ => unreachable!(),
    }
}
