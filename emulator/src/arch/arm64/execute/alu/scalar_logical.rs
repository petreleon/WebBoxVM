use super::*;

pub(in crate::arch::arm64::execute) fn exec_logical_reg(cpu: &mut Armv8Cpu, instr: Instr) {
    let n = (instr.cond & 4) != 0;
    let shift_type = instr.cond & 3;
    let mut rhs = shifted_reg_val(cpu, instr.rm, shift_type, instr.imm as u8, instr.sf);
    if n {
        rhs = !rhs;
        if !instr.sf {
            rhs &= 0xFFFFFFFF;
        }
    }
    let lhs = read_reg(cpu, instr.rn, instr.sf);
    let val = match instr.op {
        Opcode::AndReg => lhs & rhs,
        Opcode::OrrReg => lhs | rhs,
        Opcode::EorReg => lhs ^ rhs,
        Opcode::AndsReg => {
            set_nz_flags(cpu, lhs & rhs, instr.sf);
            lhs & rhs
        }
        _ => unreachable!(),
    };
    write_reg(cpu, instr.rd, val, instr.sf);
}

// ── Bitfield ──
