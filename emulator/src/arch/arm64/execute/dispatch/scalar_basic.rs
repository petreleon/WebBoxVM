use super::*;

pub(super) fn execute(cpu: &mut Armv8Cpu, instr: Instr) -> Result<Option<Flow>, &'static str> {
    match instr.op {
        Opcode::Add => write_reg(
            cpu,
            instr.rd,
            read_reg(cpu, instr.rn, instr.sf).wrapping_add(shifted_reg_val(
                cpu,
                instr.rm,
                instr.cond,
                instr.imm as u8,
                instr.sf,
            )),
            instr.sf,
        ),
        Opcode::Sub => write_reg(
            cpu,
            instr.rd,
            read_reg(cpu, instr.rn, instr.sf).wrapping_sub(shifted_reg_val(
                cpu,
                instr.rm,
                instr.cond,
                instr.imm as u8,
                instr.sf,
            )),
            instr.sf,
        ),
        Opcode::Adc | Opcode::Adcs | Opcode::Sbc | Opcode::Sbcs => exec_addsub_carry(cpu, instr),
        Opcode::Adds => {
            let lhs = read_reg(cpu, instr.rn, instr.sf);
            let rhs = shifted_reg_val(cpu, instr.rm, instr.cond, instr.imm as u8, instr.sf);
            let val = add_flags(cpu, lhs, rhs, instr.sf);
            if instr.rd != ZERO_REGISTER_INDEX {
                write_reg_sp(cpu, instr.rd, val, instr.sf);
            }
        }
        Opcode::Subs => {
            let lhs = read_reg(cpu, instr.rn, instr.sf);
            let rhs = shifted_reg_val(cpu, instr.rm, instr.cond, instr.imm as u8, instr.sf);
            let val = sub_flags(cpu, lhs, rhs, instr.sf);
            if instr.rd != ZERO_REGISTER_INDEX {
                write_reg_sp(cpu, instr.rd, val, instr.sf);
            }
        }
        Opcode::Movz => write_reg(cpu, instr.rd, instr.imm, instr.sf),
        Opcode::Movn => write_reg(cpu, instr.rd, instr.imm, instr.sf),
        Opcode::MovReg => write_reg(cpu, instr.rd, read_reg(cpu, instr.rm, instr.sf), instr.sf),
        Opcode::Sxtw => {
            let val = read_reg(cpu, instr.rn, false);
            write_reg(cpu, instr.rd, ((val as i32) as i64) as u64, true);
        }
        Opcode::Movk => {
            let hw = instr.cond as u64;
            let mask = !(0xFFFFu64 << (hw * 16));
            let old = read_reg(cpu, instr.rd, instr.sf);
            write_reg(cpu, instr.rd, (old & mask) | instr.imm, instr.sf);
        }
        Opcode::AddImm => write_reg_sp(
            cpu,
            instr.rd,
            read_base(cpu, instr.rn, instr.sf).wrapping_add(instr.imm),
            instr.sf,
        ),
        Opcode::SubImm => write_reg_sp(
            cpu,
            instr.rd,
            read_base(cpu, instr.rn, instr.sf).wrapping_sub(instr.imm),
            instr.sf,
        ),
        Opcode::AddsImm => {
            let lhs = read_base(cpu, instr.rn, instr.sf);
            let val = add_flags(cpu, lhs, instr.imm, instr.sf);
            if instr.rd != ZERO_REGISTER_INDEX {
                write_reg_sp(cpu, instr.rd, val, instr.sf);
            }
        }
        Opcode::SubsImm => {
            let lhs = read_base(cpu, instr.rn, instr.sf);
            let val = sub_flags(cpu, lhs, instr.imm, instr.sf);
            if instr.rd != ZERO_REGISTER_INDEX {
                write_reg_sp(cpu, instr.rd, val, instr.sf);
            }
        }
        Opcode::CmpImm => {
            let lhs = read_base(cpu, instr.rn, instr.sf);
            let _ = sub_flags(cpu, lhs, instr.imm, instr.sf);
        }
        Opcode::Cmp => {
            let extended = (instr.cond & 0x8) != 0;
            let lhs = if extended {
                read_base(cpu, instr.rn, instr.sf)
            } else {
                read_reg(cpu, instr.rn, instr.sf)
            };
            let rhs = if extended {
                extend_reg_val(cpu, instr.rm, instr.cond & 0x7, instr.imm as u8, instr.sf)
            } else {
                shifted_reg_val(cpu, instr.rm, instr.cond, instr.imm as u8, instr.sf)
            };
            let _ = sub_flags(cpu, lhs, rhs, instr.sf);
        }
        Opcode::Adr => write_reg(cpu, instr.rd, branch_target(cpu.regs.pc, instr.imm), true),
        Opcode::Adrp => {
            let page = cpu.regs.pc & !PAGE_OFFSET_MASK;
            write_reg(cpu, instr.rd, (page as i64 + instr.imm as i64) as u64, true);
        }
        _ => return Ok(None),
    }
    Ok(Some(Flow::Advance))
}
