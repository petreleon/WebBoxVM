//! Execute decoded AArch64 instructions.

use super::opcodes::{Instr, Opcode};
use super::helpers::{cond_taken, read_reg, read_base, write_reg, write_reg_sp};
use super::Armv8Cpu;
use crate::bus::SystemBus;
use crate::arm64::mmu::translate;

/// Mutate CPU and bus state according to the decoded instruction.
pub fn execute(cpu: &mut Armv8Cpu, bus: &mut SystemBus, instr: Instr) -> Result<(), &'static str> {
    match instr.op {
        Opcode::Add  => write_reg_sp(cpu, instr.rd, read_reg(cpu, instr.rn, instr.sf).wrapping_add(shifted_reg_val(cpu, instr.rm, instr.cond, instr.imm as u8, instr.sf)), instr.sf),
        Opcode::Sub  => write_reg_sp(cpu, instr.rd, read_reg(cpu, instr.rn, instr.sf).wrapping_sub(shifted_reg_val(cpu, instr.rm, instr.cond, instr.imm as u8, instr.sf)), instr.sf),
        Opcode::Movz => write_reg(cpu, instr.rd, instr.imm, instr.sf),
        Opcode::Movk => {
            let hw = instr.imm.trailing_zeros() / 16;
            let mask = !(0xFFFFu64 << (hw * 16));
            let old = read_reg(cpu, instr.rd, instr.sf);
            write_reg(cpu, instr.rd, (old & mask) | instr.imm, instr.sf);
        }
        Opcode::Movn => write_reg(cpu, instr.rd, instr.imm, instr.sf),
        Opcode::MovReg => write_reg(cpu, instr.rd, read_reg(cpu, instr.rm, instr.sf), instr.sf),
        Opcode::Sxtw => {
            let val = read_reg(cpu, instr.rn, false);
            let signed = ((val as i32) as i64) as u64;
            write_reg(cpu, instr.rd, signed, true);
        }
        Opcode::AddImm => write_reg_sp(cpu, instr.rd, read_base(cpu, instr.rn, instr.sf).wrapping_add(instr.imm), instr.sf),
        Opcode::SubImm => write_reg_sp(cpu, instr.rd, read_base(cpu, instr.rn, instr.sf).wrapping_sub(instr.imm), instr.sf),
        Opcode::AddsImm => {
            let lhs = read_base(cpu, instr.rn, instr.sf);
            let val = add_flags(cpu, lhs, instr.imm, instr.sf);
            write_reg_sp(cpu, instr.rd, val, instr.sf);
        }
        Opcode::SubsImm => {
            let lhs = read_base(cpu, instr.rn, instr.sf);
            let val = sub_flags(cpu, lhs, instr.imm, instr.sf);
            write_reg_sp(cpu, instr.rd, val, instr.sf);
        }
        Opcode::Adr    => write_reg(cpu, instr.rd, (cpu.regs.pc as i64 + instr.imm as i64) as u64, true),
        Opcode::Adrp   => {
            let page = cpu.regs.pc & !0xFFF;
            write_reg(cpu, instr.rd, (page as i64 + instr.imm as i64) as u64, true);
        }
        Opcode::Ldr => {
            let va = addr_with_offset(cpu, instr.rn, instr.imm)?;
            let addr = translate(&cpu.sys, &mut cpu.tlb, &bus.mem, va).map_err(|_| "LDR translation fault")?;
            let size = if instr.size != 0 { instr.size } else if instr.sf { 8 } else { 4 };
            let val = bus.read(addr, size).ok_or("LDR bus fault")?;
            write_reg(cpu, instr.rd, val, instr.sf);
        }
        Opcode::LdrLit => {
            let va = (cpu.regs.pc as i64 + instr.imm as i64) as u64;
            let addr = translate(&cpu.sys, &mut cpu.tlb, &bus.mem, va).map_err(|_| "LDR literal translation fault")?;
            let size = if instr.sf { 8 } else { 4 };
            let val = bus.read(addr, size).ok_or("LDR literal bus fault")?;
            write_reg(cpu, instr.rd, val, instr.sf);
        }
        Opcode::Str => {
            let va = addr_with_offset(cpu, instr.rn, instr.imm)?;
            let addr = translate(&cpu.sys, &mut cpu.tlb, &bus.mem, va).map_err(|_| "STR translation fault")?;
            let val = read_reg(cpu, instr.rd, instr.sf);
            let size = if instr.size != 0 { instr.size } else if instr.sf { 8 } else { 4 };
            bus.write(addr, size, val);
        }
        Opcode::Ldp => exec_ldp_stp(cpu, bus, instr, true)?,
        Opcode::Stp => exec_ldp_stp(cpu, bus, instr, false)?,
        Opcode::B  => { cpu.regs.pc = (cpu.regs.pc as i64 + instr.imm as i64) as u64; return Ok(()); }
        Opcode::Bl => {
            cpu.regs.set_x(30, cpu.regs.pc + 4);
            cpu.regs.pc = (cpu.regs.pc as i64 + instr.imm as i64) as u64;
            return Ok(());
        }
        Opcode::Blr => {
            cpu.regs.set_x(30, cpu.regs.pc + 4);
            cpu.regs.pc = read_reg(cpu, instr.rn, true);
            return Ok(());
        }
        Opcode::Br  => { cpu.regs.pc = read_reg(cpu, instr.rn, true); return Ok(()); }
        Opcode::Ret => { cpu.regs.pc = read_reg(cpu, instr.rn, true); return Ok(()); }
        Opcode::Cbz  => {
            if read_reg(cpu, instr.rd, instr.sf) == 0 {
                cpu.regs.pc = (cpu.regs.pc as i64 + instr.imm as i64) as u64;
                return Ok(());
            }
        }
        Opcode::Cbnz => {
            if read_reg(cpu, instr.rd, instr.sf) != 0 {
                cpu.regs.pc = (cpu.regs.pc as i64 + instr.imm as i64) as u64;
                return Ok(());
            }
        }
        Opcode::BCond => {
            if cond_taken(cpu, instr.cond) {
                cpu.regs.pc = (cpu.regs.pc as i64 + instr.imm as i64) as u64;
                return Ok(());
            }
        }
        Opcode::Csel => {
            let val = if cond_taken(cpu, instr.cond) { read_reg(cpu, instr.rn, instr.sf) } else { read_reg(cpu, instr.rm, instr.sf) };
            write_reg(cpu, instr.rd, val, instr.sf);
        }
        Opcode::Ccmp => {
            if cond_taken(cpu, instr.cond) {
                let lhs = read_reg(cpu, instr.rn, instr.sf);
                let rhs = read_reg(cpu, instr.rm, instr.sf);
                let _ = sub_flags(cpu, lhs, rhs, instr.sf);
            }
        }
        Opcode::Tbz => {
            let val = read_reg(cpu, instr.rd, true);
            let bit = instr.cond as u64;
            if ((val >> bit) & 1) == 0 {
                cpu.regs.pc = (cpu.regs.pc as i64 + instr.imm as i64) as u64;
                return Ok(());
            }
        }
        Opcode::Tbnz => {
            let val = read_reg(cpu, instr.rd, true);
            let bit = instr.cond as u64;
            if ((val >> bit) & 1) != 0 {
                cpu.regs.pc = (cpu.regs.pc as i64 + instr.imm as i64) as u64;
                return Ok(());
            }
        }
        Opcode::Cmp => {
            let lhs = read_reg(cpu, instr.rn, instr.sf);
            let rhs = shifted_reg_val(cpu, instr.rm, instr.cond, instr.imm as u8, instr.sf);
            let _ = sub_flags(cpu, lhs, rhs, instr.sf);
        }
        Opcode::CmpImm => {
            let lhs = read_reg(cpu, instr.rn, instr.sf);
            let _ = sub_flags(cpu, lhs, instr.imm, instr.sf);
        }
        Opcode::Mrs => {
            let el = cpu.pstate.el();
            let val = cpu.sys.read_sys_reg(instr.imm as u16, el);
            write_reg(cpu, instr.rd, val, true);
        }
        Opcode::Msr => {
            let val = read_reg(cpu, instr.rd, true);
            let sysreg_id = instr.imm as u16;
            cpu.sys.write_sys_reg(sysreg_id, val);
            match sysreg_id {
                0x4100 | 0x4101 | 0x4102 => cpu.tlb.invalidate_all(),
                _ => {}
            }
        }
        Opcode::Tlbi => {
            cpu.tlb.invalidate_all();
        }
        Opcode::Madd => {
            let sf_src = instr.size == 0 && instr.sf;
            let n = read_reg(cpu, instr.rn, sf_src);
            let m = read_reg(cpu, instr.rm, sf_src);
            let a = read_reg(cpu, instr.cond, instr.sf);

            let val = match instr.size {
                0 => {
                    if instr.sf {
                        a.wrapping_add(n.wrapping_mul(m))
                    } else {
                        let res = (a as u32).wrapping_add((n as u32).wrapping_mul(m as u32));
                        res as u64
                    }
                }
                1 => {
                    let prod = (n as u32 as u64).wrapping_mul(m as u32 as u64);
                    a.wrapping_add(prod)
                }
                2 => {
                    let n_signed = (n as u32 as i32) as i64;
                    let m_signed = (m as u32 as i32) as i64;
                    let prod = n_signed.wrapping_mul(m_signed) as u64;
                    a.wrapping_add(prod)
                }
                _ => return Err("Invalid Madd size"),
            };
            write_reg(cpu, instr.rd, val, instr.sf);
        }
        Opcode::Msub => {
            let sf_src = instr.size == 0 && instr.sf;
            let n = read_reg(cpu, instr.rn, sf_src);
            let m = read_reg(cpu, instr.rm, sf_src);
            let a = read_reg(cpu, instr.cond, instr.sf);

            let val = match instr.size {
                0 => {
                    if instr.sf {
                        a.wrapping_sub(n.wrapping_mul(m))
                    } else {
                        let res = (a as u32).wrapping_sub((n as u32).wrapping_mul(m as u32));
                        res as u64
                    }
                }
                1 => {
                    let prod = (n as u32 as u64).wrapping_mul(m as u32 as u64);
                    a.wrapping_sub(prod)
                }
                2 => {
                    let n_signed = (n as u32 as i32) as i64;
                    let m_signed = (m as u32 as i32) as i64;
                    let prod = n_signed.wrapping_mul(m_signed) as u64;
                    a.wrapping_sub(prod)
                }
                _ => return Err("Invalid Msub size"),
            };
            write_reg(cpu, instr.rd, val, instr.sf);
        }
        Opcode::AndImm => {
            let val = read_reg(cpu, instr.rn, instr.sf) & instr.imm;
            write_reg(cpu, instr.rd, val, instr.sf);
        }
        Opcode::OrrImm => {
            let val = read_reg(cpu, instr.rn, instr.sf) | instr.imm;
            write_reg(cpu, instr.rd, val, instr.sf);
        }
        Opcode::EorImm => {
            let val = read_reg(cpu, instr.rn, instr.sf) ^ instr.imm;
            write_reg(cpu, instr.rd, val, instr.sf);
        }
        Opcode::AndsImm => {
            let val = read_reg(cpu, instr.rn, instr.sf) & instr.imm;
            let sign_bit = if instr.sf { 63 } else { 31 };
            cpu.pstate.set_nzcv(((val >> sign_bit) & 1) != 0, if instr.sf { val == 0 } else { (val as u32) == 0 }, false, false);
            write_reg(cpu, instr.rd, val, instr.sf);
        }
        Opcode::Sbfm | Opcode::Bfm | Opcode::Ubfm => {
            let size = if instr.sf { 64 } else { 32 };
            let r = instr.rm as u32; // immr
            let s = instr.imm as u32; // imms
            let src = read_reg(cpu, instr.rn, instr.sf);

            let val = if instr.op == Opcode::Ubfm {
                let result = if s >= r {
                    let len = s - r + 1;
                    let mask = if len >= 64 { !0 } else { (1u64 << len) - 1 };
                    (src >> r) & mask
                } else {
                    let len = s + 1;
                    let mask = if len >= 64 { !0 } else { (1u64 << len) - 1 };
                    let shift = size - r;
                    (src & mask) << shift
                };
                if !instr.sf { result & 0xFFFFFFFF } else { result }
            } else if instr.op == Opcode::Sbfm {
                let result = if s >= r {
                    let len = s - r + 1;
                    let mask = if len >= 64 { !0 } else { (1u64 << len) - 1 };
                    let extracted = (src >> r) & mask;
                    let sign_bit = s - r;
                    if sign_bit < 63 && (extracted & (1u64 << sign_bit)) != 0 {
                        let extend_mask = !((1u64 << (sign_bit + 1)) - 1);
                        extracted | (extend_mask & if instr.sf { !0 } else { 0xFFFFFFFF })
                    } else {
                        extracted
                    }
                } else {
                    let len = s + 1;
                    let mask = if len >= 64 { !0 } else { (1u64 << len) - 1 };
                    let shift = size - r;
                    let extracted = (src & mask) << shift;
                    let sign_bit = shift + s;
                    if sign_bit < 63 && (extracted & (1u64 << sign_bit)) != 0 {
                        let extend_mask = !((1u64 << (sign_bit + 1)) - 1);
                        extracted | (extend_mask & if instr.sf { !0 } else { 0xFFFFFFFF })
                    } else {
                        extracted
                    }
                };
                if !instr.sf { result & 0xFFFFFFFF } else { result }
            } else {
                let dst = read_reg(cpu, instr.rd, instr.sf);
                let result = if s >= r {
                    let len = s - r + 1;
                    let mask = if len >= 64 { !0 } else { (1u64 << len) - 1 };
                    let dst_mask = !(mask << r);
                    (dst & dst_mask) | ((src & mask) << r)
                } else {
                    let len = s + 1;
                    let mask = if len >= 64 { !0 } else { (1u64 << len) - 1 };
                    let shift = size - r;
                    let dst_mask = !(mask << shift);
                    (dst & dst_mask) | ((src & mask) << shift)
                };
                if !instr.sf { result & 0xFFFFFFFF } else { result }
            };
            write_reg(cpu, instr.rd, val, instr.sf);
        }
        Opcode::AndReg | Opcode::OrrReg | Opcode::EorReg | Opcode::AndsReg => {
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
                    let res = lhs & rhs;
                    let sign_bit = if instr.sf { 63 } else { 31 };
                    cpu.pstate.set_nzcv(((res >> sign_bit) & 1) != 0, if instr.sf { res == 0 } else { (res as u32) == 0 }, false, false);
                    res
                }
                _ => unreachable!(),
            };
            write_reg(cpu, instr.rd, val, instr.sf);
        }
        Opcode::AddExt => {
            let lhs = read_base(cpu, instr.rn, instr.sf);
            let rhs = extend_reg_val(cpu, instr.rm, instr.cond, instr.imm as u8, instr.sf);
            write_reg_sp(cpu, instr.rd, lhs.wrapping_add(rhs), instr.sf);
        }
        Opcode::SubExt => {
            let lhs = read_base(cpu, instr.rn, instr.sf);
            let rhs = extend_reg_val(cpu, instr.rm, instr.cond, instr.imm as u8, instr.sf);
            write_reg_sp(cpu, instr.rd, lhs.wrapping_sub(rhs), instr.sf);
        }
        Opcode::Nop | Opcode::NopBarrier => {}
    }
    cpu.regs.pc += 4;
    Ok(())
}

fn extend_reg_val(cpu: &Armv8Cpu, rm: u8, option: u8, shift: u8, sf: bool) -> u64 {
    let mut val = read_reg(cpu, rm, if option == 3 || option == 7 { sf } else { option >= 2 });
    val = match option {
        0 => (val as u8) as u64,
        1 => (val as u16) as u64,
        2 => (val as u32) as u64,
        3 => val,
        4 => ((val as i8) as i64) as u64,
        5 => ((val as i16) as i64) as u64,
        6 => ((val as i32) as i64) as u64,
        7 => val,
        _ => val,
    };
    if sf {
        val << shift
    } else {
        ((val as u32) << shift) as u64
    }
}

fn add_flags(cpu: &mut Armv8Cpu, lhs: u64, rhs: u64, sf: bool) -> u64 {
    let val = lhs.wrapping_add(rhs);
    let sign_bit = if sf { 63 } else { 31 };
    let n = ((val >> sign_bit) & 1) != 0;
    let z = if sf { val == 0 } else { (val as u32) == 0 };
    let c = if sf {
        val < lhs
    } else {
        (val as u32) < (lhs as u32)
    };
    let sign_mask = 1u64 << sign_bit;
    let lhs_sign = lhs & sign_mask;
    let rhs_sign = rhs & sign_mask;
    let res_sign = val & sign_mask;
    let v = (lhs_sign == rhs_sign) && (lhs_sign != res_sign);
    cpu.pstate.set_nzcv(n, z, c, v);
    val
}

fn sub_flags(cpu: &mut Armv8Cpu, lhs: u64, rhs: u64, sf: bool) -> u64 {
    let val = lhs.wrapping_sub(rhs);
    let sign_bit = if sf { 63 } else { 31 };
    let n = ((val >> sign_bit) & 1) != 0;
    let z = if sf { val == 0 } else { (val as u32) == 0 };
    let c = if sf { lhs >= rhs } else { (lhs as u32) >= (rhs as u32) };
    let sign_mask = 1u64 << sign_bit;
    let lhs_sign = lhs & sign_mask;
    let rhs_sign = rhs & sign_mask;
    let res_sign = val & sign_mask;
    let v = (lhs_sign != rhs_sign) && (lhs_sign != res_sign);
    cpu.pstate.set_nzcv(n, z, c, v);
    val
}

fn shifted_reg_val(cpu: &Armv8Cpu, rm: u8, shift_type: u8, amount: u8, sf: bool) -> u64 {
    let val = read_reg(cpu, rm, sf);
    let amount = amount as u32;
    if amount == 0 {
        return val;
    }
    match shift_type {
        0 => { // LSL
            if sf { val << amount } else { ((val as u32) << amount) as u64 }
        }
        1 => { // LSR
            if sf { val >> amount } else { ((val as u32) >> amount) as u64 }
        }
        2 => { // ASR
            if sf {
                ((val as i64) >> amount) as u64
            } else {
                (((val as u32) as i32) >> amount) as u64
            }
        }
        3 => { // ROR
            if sf {
                val.rotate_right(amount)
            } else {
                (val as u32).rotate_right(amount) as u64
            }
        }
        _ => val,
    }
}

fn exec_ldp_stp(cpu: &mut Armv8Cpu, bus: &mut SystemBus, instr: Instr, is_load: bool) -> Result<(), &'static str> {
    let base = read_base(cpu, instr.rn, true);
    let size = if instr.size != 0 { instr.size as u64 } else if instr.sf { 8u64 } else { 4u64 };
    let (va, new_base) = match instr.cond {
        1 => (base, (base as i64).wrapping_add(instr.imm as i64) as u64),
        3 => {
            let b = (base as i64).wrapping_add(instr.imm as i64) as u64;
            (b, b)
        }
        _ => ((base as i64).wrapping_add(instr.imm as i64) as u64, base),
    };
    let addr = translate(&cpu.sys, &mut cpu.tlb, &bus.mem, va).map_err(|_| "LDP translation fault")?;
    let addr2 = translate(&cpu.sys, &mut cpu.tlb, &bus.mem, va + size).map_err(|_| "LDP translation fault")?;
    if is_load {
        write_reg(cpu, instr.rd, bus.read(addr, size as u8).ok_or("LDP bus fault")?, instr.sf);
        write_reg(cpu, instr.rm, bus.read(addr2, size as u8).ok_or("LDP bus fault")?, instr.sf);
    } else {
        bus.write(addr, size as u8, read_reg(cpu, instr.rd, instr.sf));
        bus.write(addr2, size as u8, read_reg(cpu, instr.rm, instr.sf));
    }
    if new_base != base {
        write_reg_sp(cpu, instr.rn, new_base, true);
    }
    Ok(())
}

fn addr_with_offset(cpu: &Armv8Cpu, base: u8, offset: u64) -> Result<u64, &'static str> {
    let base_addr = if base >= 31 { cpu.regs.sp } else { cpu.regs.x(base) };
    Ok(base_addr.wrapping_add(offset))
}

#[cfg(test)]
mod tests;
