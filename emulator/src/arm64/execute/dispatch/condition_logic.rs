use super::*;

pub(super) fn execute(cpu: &mut Armv8Cpu, instr: Instr) -> Result<Option<Flow>, &'static str> {
    match instr.op {
        Opcode::Csel => write_reg(
            cpu,
            instr.rd,
            if cond_taken(cpu, instr.cond) {
                read_reg(cpu, instr.rn, instr.sf)
            } else {
                read_reg(cpu, instr.rm, instr.sf)
            },
            instr.sf,
        ),
        Opcode::Csinc => write_reg(
            cpu,
            instr.rd,
            if cond_taken(cpu, instr.cond) {
                read_reg(cpu, instr.rn, instr.sf)
            } else {
                read_reg(cpu, instr.rm, instr.sf).wrapping_add(1)
            },
            instr.sf,
        ),
        Opcode::Csinv => write_reg(
            cpu,
            instr.rd,
            if cond_taken(cpu, instr.cond) {
                read_reg(cpu, instr.rn, instr.sf)
            } else {
                !read_reg(cpu, instr.rm, instr.sf)
            },
            instr.sf,
        ),
        Opcode::Csneg => write_reg(
            cpu,
            instr.rd,
            if cond_taken(cpu, instr.cond) {
                read_reg(cpu, instr.rn, instr.sf)
            } else {
                0u64.wrapping_sub(read_reg(cpu, instr.rm, instr.sf))
            },
            instr.sf,
        ),
        Opcode::Ccmp | Opcode::Ccmn => exec_condcmp(cpu, instr),
        Opcode::AndImm => write_reg(
            cpu,
            instr.rd,
            read_reg(cpu, instr.rn, instr.sf) & instr.imm,
            instr.sf,
        ),
        Opcode::OrrImm => write_reg(
            cpu,
            instr.rd,
            read_reg(cpu, instr.rn, instr.sf) | instr.imm,
            instr.sf,
        ),
        Opcode::EorImm => write_reg(
            cpu,
            instr.rd,
            read_reg(cpu, instr.rn, instr.sf) ^ instr.imm,
            instr.sf,
        ),
        Opcode::AndsImm => {
            let val = read_reg(cpu, instr.rn, instr.sf) & instr.imm;
            set_nz_flags(cpu, val, instr.sf);
            write_reg(cpu, instr.rd, val, instr.sf);
        }
        Opcode::AndReg | Opcode::OrrReg | Opcode::EorReg | Opcode::AndsReg => {
            exec_logical_reg(cpu, instr)
        }
        Opcode::Sbfm | Opcode::Bfm | Opcode::Ubfm => exec_bitfield(cpu, instr),
        _ => return Ok(None),
    }
    Ok(Some(Flow::Advance))
}
