use super::*;

pub(super) fn is_opcode(op: Opcode) -> bool {
    matches!(
        op,
        Opcode::SveCnt
            | Opcode::SveRdvl
            | Opcode::SveRdsvl
            | Opcode::SveAddvl
            | Opcode::SveAddsvl
            | Opcode::SveAddpl
            | Opcode::SveAddspl
            | Opcode::SveIncScalar
            | Opcode::SveDecScalar
    )
}

pub(super) fn execute(cpu: &mut Armv8Cpu, instr: Instr) {
    match instr.op {
        Opcode::SveCnt => exec_cnt(cpu, instr),
        Opcode::SveRdvl | Opcode::SveRdsvl => exec_rd_len(cpu, instr),
        Opcode::SveAddvl | Opcode::SveAddsvl | Opcode::SveAddpl | Opcode::SveAddspl => {
            exec_add_len(cpu, instr)
        }
        Opcode::SveIncScalar | Opcode::SveDecScalar => exec_inc_dec_scalar(cpu, instr),
        _ => unreachable!("not an SVE scalar count opcode"),
    }
}

fn exec_cnt(cpu: &mut Armv8Cpu, instr: Instr) {
    let elements = (cpu.sve_vl_bytes as u64) / instr.size as u64;
    let count = sve_pred_count(instr.cond, elements).wrapping_mul(instr.imm);
    write_reg(cpu, instr.rd, count, true);
}

fn exec_rd_len(cpu: &mut Armv8Cpu, instr: Instr) {
    let scale_bytes = if instr.op == Opcode::SveRdvl {
        cpu.sve_vl_bytes as i64
    } else {
        cpu.sme_svl_bytes as i64
    };
    let result = (instr.imm as i64).wrapping_mul(scale_bytes) as u64;
    write_reg(cpu, instr.rd, result, true);
}

fn exec_add_len(cpu: &mut Armv8Cpu, instr: Instr) {
    let scale_bytes = match instr.op {
        Opcode::SveAddvl => cpu.sve_vl_bytes as i64,
        Opcode::SveAddsvl => cpu.sme_svl_bytes as i64,
        Opcode::SveAddpl => sve_pl_bytes(cpu) as i64,
        _ => cpu.sme_svl_bytes as i64 / 8,
    };
    let offset = (instr.imm as i64).wrapping_mul(scale_bytes) as u64;
    let result = read_base(cpu, instr.rn, true).wrapping_add(offset);
    write_reg_sp(cpu, instr.rd, result, true);
}

fn exec_inc_dec_scalar(cpu: &mut Armv8Cpu, instr: Instr) {
    let elements = (cpu.sve_vl_bytes as u64) / instr.size as u64;
    let count = sve_pred_count(instr.cond, elements).wrapping_mul(instr.imm);
    let old = read_reg(cpu, instr.rd, true);
    let result = if instr.op == Opcode::SveIncScalar {
        old.wrapping_add(count)
    } else {
        old.wrapping_sub(count)
    };
    write_reg(cpu, instr.rd, result, true);
}
