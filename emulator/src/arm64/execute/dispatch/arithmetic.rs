use super::*;

pub(super) fn execute(cpu: &mut Armv8Cpu, instr: Instr) -> Result<Option<Flow>, &'static str> {
    match instr.op {
        Opcode::AddExt => write_reg_sp(
            cpu,
            instr.rd,
            read_base(cpu, instr.rn, instr.sf).wrapping_add(extend_reg_val(
                cpu,
                instr.rm,
                instr.cond,
                instr.imm as u8,
                instr.sf,
            )),
            instr.sf,
        ),
        Opcode::SubExt => write_reg_sp(
            cpu,
            instr.rd,
            read_base(cpu, instr.rn, instr.sf).wrapping_sub(extend_reg_val(
                cpu,
                instr.rm,
                instr.cond,
                instr.imm as u8,
                instr.sf,
            )),
            instr.sf,
        ),
        Opcode::AddsExt => {
            let lhs = read_base(cpu, instr.rn, instr.sf);
            let rhs = extend_reg_val(cpu, instr.rm, instr.cond, instr.imm as u8, instr.sf);
            let val = add_flags(cpu, lhs, rhs, instr.sf);
            if instr.rd != ZERO_REGISTER_INDEX {
                write_reg_sp(cpu, instr.rd, val, instr.sf);
            }
        }
        Opcode::SubsExt => {
            let lhs = read_base(cpu, instr.rn, instr.sf);
            let rhs = extend_reg_val(cpu, instr.rm, instr.cond, instr.imm as u8, instr.sf);
            let val = sub_flags(cpu, lhs, rhs, instr.sf);
            if instr.rd != ZERO_REGISTER_INDEX {
                write_reg_sp(cpu, instr.rd, val, instr.sf);
            }
        }
        Opcode::Madd => exec_madd(cpu, instr),
        Opcode::Msub => exec_msub(cpu, instr),
        Opcode::Umulh => {
            let n = read_reg(cpu, instr.rn, true);
            let m = read_reg(cpu, instr.rm, true);
            write_reg(
                cpu,
                instr.rd,
                ((n as u128).wrapping_mul(m as u128) >> 64) as u64,
                true,
            );
        }
        Opcode::Smulh => {
            let n = read_reg(cpu, instr.rn, true) as i64;
            let m = read_reg(cpu, instr.rm, true) as i64;
            write_reg(
                cpu,
                instr.rd,
                ((n as i128).wrapping_mul(m as i128) >> 64) as u64,
                true,
            );
        }
        Opcode::Udiv => exec_div(cpu, instr, false),
        Opcode::Sdiv => exec_div(cpu, instr, true),
        Opcode::Lslv => exec_variable_shift(cpu, instr, ShiftDir::Left),
        Opcode::Lsrv => exec_variable_shift(cpu, instr, ShiftDir::Right),
        Opcode::Asrv => exec_variable_shift(cpu, instr, ShiftDir::ArithRight),
        Opcode::Rorv => exec_variable_shift(cpu, instr, ShiftDir::RotateRight),
        Opcode::Extr => exec_extract(cpu, instr),
        Opcode::Rev => exec_rev(cpu, instr),
        Opcode::Rev32 => {
            let val = read_reg(cpu, instr.rn, true);
            let low = (val as u32).swap_bytes() as u64;
            let high = ((val >> 32) as u32).swap_bytes() as u64;
            write_reg(cpu, instr.rd, (high << 32) | low, true);
        }
        Opcode::Rev16 => exec_rev16(cpu, instr),
        Opcode::Rbit => exec_rbit(cpu, instr),
        Opcode::Clz => exec_clz(cpu, instr),
        Opcode::Cls => exec_cls(cpu, instr),
        Opcode::Ctz => exec_ctz(cpu, instr),
        Opcode::Cnt => exec_cnt(cpu, instr),
        Opcode::Abs => exec_abs(cpu, instr),
        Opcode::Crc32 => exec_crc32(cpu, instr),
        Opcode::Crc32c => exec_crc32c(cpu, instr),
        _ => return Ok(None),
    }
    Ok(Some(Flow::Advance))
}
