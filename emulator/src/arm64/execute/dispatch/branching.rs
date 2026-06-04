use super::*;

pub(super) fn execute(cpu: &mut Armv8Cpu, instr: Instr) -> Result<Option<Flow>, &'static str> {
    match instr.op {
        Opcode::B => branch(cpu, instr.imm)?,
        Opcode::Bl => branch_link(cpu, instr.imm)?,
        Opcode::Blr => branch_link_reg(cpu, instr.rn)?,
        Opcode::Br => branch_reg(cpu, instr.rn)?,
        Opcode::Ret => branch_reg(cpu, instr.rn)?,
        Opcode::Cbz => {
            if read_reg(cpu, instr.rd, instr.sf) == 0 {
                branch(cpu, instr.imm)?;
                return Ok(Some(Flow::Return));
            }
            return Ok(Some(Flow::Advance));
        }
        Opcode::Cbnz => {
            if read_reg(cpu, instr.rd, instr.sf) != 0 {
                branch(cpu, instr.imm)?;
                return Ok(Some(Flow::Return));
            }
            return Ok(Some(Flow::Advance));
        }
        Opcode::BCond => {
            if cond_taken(cpu, instr.cond) {
                branch(cpu, instr.imm)?;
                return Ok(Some(Flow::Return));
            }
            return Ok(Some(Flow::Advance));
        }
        Opcode::Tbz => {
            if (read_reg(cpu, instr.rd, instr.sf) >> (instr.cond as u64)) & 1 == 0 {
                branch(cpu, instr.imm)?;
                return Ok(Some(Flow::Return));
            }
            return Ok(Some(Flow::Advance));
        }
        Opcode::Tbnz => {
            if (read_reg(cpu, instr.rd, instr.sf) >> (instr.cond as u64)) & 1 != 0 {
                branch(cpu, instr.imm)?;
                return Ok(Some(Flow::Return));
            }
            return Ok(Some(Flow::Advance));
        }
        _ => return Ok(None),
    }
    Ok(Some(Flow::Return))
}
