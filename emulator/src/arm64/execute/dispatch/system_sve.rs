use super::*;

pub(super) fn execute(
    cpu: &mut Armv8Cpu,
    bus: &mut SystemBus,
    instr: Instr,
) -> Result<Option<Flow>, &'static str> {
    match instr.op {
        Opcode::Mrs => {
            let sysreg_id = instr.imm as u16;
            let val = if sysreg_id == SYSREG_DAIF {
                cpu.pstate.daif()
            } else {
                cpu.sys.read_sys_reg(sysreg_id, cpu.pstate.el())
            };
            write_reg(cpu, instr.rd, val, true);
        }
        Opcode::Msr => exec_msr(cpu, instr),
        Opcode::SveCnt => {
            let elements = (cpu.sve_vl_bytes as u64) / instr.size as u64;
            let count = sve_pred_count(instr.cond, elements).wrapping_mul(instr.imm);
            write_reg(cpu, instr.rd, count, true);
        }
        Opcode::SveAddvl | Opcode::SveAddsvl => {
            let vl_bytes = if instr.op == Opcode::SveAddvl {
                cpu.sve_vl_bytes as i64
            } else {
                cpu.sme_svl_bytes as i64
            };
            let offset = (instr.imm as i64).wrapping_mul(vl_bytes) as u64;
            let result = read_base(cpu, instr.rn, true).wrapping_add(offset);
            write_reg_sp(cpu, instr.rd, result, true);
        }
        Opcode::SvePtrue => exec_sve_ptrue(cpu, instr),
        Opcode::SvePtest => exec_sve_ptest(cpu, instr),
        Opcode::SvePredAnd | Opcode::SvePredOrr => exec_sve_pred_logical(cpu, instr),
        Opcode::SveMovprfx => exec_sve_movprfx(cpu, instr),
        Opcode::SveDupGpr => exec_sve_dup_gpr(cpu, instr),
        Opcode::SveAddVec | Opcode::SveSubVec => exec_sve_int_binary(cpu, instr),
        Opcode::SveOrrVec | Opcode::SveEorVec => exec_sve_logical_binary(cpu, instr),
        Opcode::SveAndImm | Opcode::SveOrrImm | Opcode::SveEorImm | Opcode::SveDupm => {
            exec_sve_logical_imm(cpu, instr)
        }
        Opcode::SveFpDupImm => exec_sve_fp_dup_imm(cpu, instr),
        Opcode::SveFpAdd
        | Opcode::SveFpSub
        | Opcode::SveFpMul
        | Opcode::SveFpSubr
        | Opcode::SveFpMulImm => exec_sve_fp_binary(cpu, instr),
        Opcode::SveFpFmla | Opcode::SveFpFmls | Opcode::SveFpFmad | Opcode::SveFpFmsb => {
            exec_sve_fp_fused(cpu, instr)
        }
        Opcode::SveFpFmlaIndex | Opcode::SveFpFmlsIndex | Opcode::SveFpMulIndex => {
            exec_sve_fp_indexed(cpu, instr)
        }
        Opcode::SveSel => exec_sve_sel(cpu, instr),
        Opcode::SveLdr | Opcode::SveStr => exec_sve_ldr_str(cpu, bus, instr)?,
        Opcode::SveLd1b | Opcode::SveLd1rw | Opcode::SveLd1rqw => {
            exec_sve_contiguous_load(cpu, bus, instr)?
        }
        Opcode::SveSt1b => exec_sve_st1b(cpu, bus, instr)?,
        Opcode::SveLd1rd | Opcode::SveLd1rqd | Opcode::SveLd1d | Opcode::SveSt1d => {
            exec_sve_ld1_st1_d(cpu, bus, instr)?
        }
        _ => return Ok(None),
    }
    Ok(Some(Flow::Advance))
}
