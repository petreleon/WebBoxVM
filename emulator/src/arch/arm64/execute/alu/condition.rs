use super::*;

pub(in crate::arch::arm64::execute) fn exec_condcmp(cpu: &mut Armv8Cpu, instr: Instr) {
    if cond_taken(cpu, instr.cond) {
        let lhs = read_reg(cpu, instr.rn, instr.sf);
        let rhs = if instr.size == 1 {
            instr.rm as u64
        } else {
            read_reg(cpu, instr.rm, instr.sf)
        };
        if instr.op == Opcode::Ccmn {
            let _ = add_flags(cpu, lhs, rhs, instr.sf);
        } else {
            let _ = sub_flags(cpu, lhs, rhs, instr.sf);
        }
    } else {
        let n = (instr.imm & 8) != 0;
        let z = (instr.imm & 4) != 0;
        let c = (instr.imm & 2) != 0;
        let v = (instr.imm & 1) != 0;
        cpu.pstate.set_nzcv(n, z, c, v);
    }
}

// ── Multiply ──
