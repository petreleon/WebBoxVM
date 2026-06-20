use super::*;

pub(in crate::arch::arm64::execute) fn exec_pointer_subtract(cpu: &mut Armv8Cpu, instr: Instr) {
    let lhs = pointer_operand(cpu, instr.rn);
    let rhs = pointer_operand(cpu, instr.rm);
    let result = if instr.op == Opcode::Subps {
        sub_flags(cpu, lhs, rhs, true)
    } else {
        lhs.wrapping_sub(rhs)
    };
    write_reg(cpu, instr.rd, result, true);
}

fn pointer_operand(cpu: &Armv8Cpu, reg: u8) -> u64 {
    sign_extend_56(read_base(cpu, reg, true))
}

fn sign_extend_56(value: u64) -> u64 {
    ((value << 8) as i64 >> 8) as u64
}
