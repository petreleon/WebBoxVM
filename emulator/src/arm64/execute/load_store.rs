//! Load/Store instruction execution.

use crate::arm64::helpers::{read_reg, write_reg, write_reg_sp, read_base};
use crate::arm64::mmu::translate;
use crate::arm64::Armv8Cpu;
use crate::bus::SystemBus;
use crate::constants::*;
use super::{Instr, Opcode, branch_target};

fn compute_ldst_va(cpu: &Armv8Cpu, instr: &Instr) -> (u64, Option<u64>) {
    if instr.rm != 0xFF {
        let base = base_addr(cpu, instr.rn);
        let offset_val = read_reg(cpu, instr.rm, true);
        let extended = apply_extension(offset_val, instr.cond);
        let shift = if instr.imm == 1 { instr.size.trailing_zeros() as u8 } else { 0 };
        (base.wrapping_add(extended << shift), None)
    } else {
        let base = base_addr(cpu, instr.rn);
        let (va, wb) = match instr.cond {
            1 => (base, Some(base.wrapping_add(instr.imm))),
            3 => { let b = base.wrapping_add(instr.imm); (b, Some(b)) },
            _ => (base.wrapping_add(instr.imm), None),
        };
        (va, wb)
    }
}

fn base_addr(cpu: &Armv8Cpu, rn: u8) -> u64 {
    if rn == SP_REGISTER_INDEX { cpu.regs.sp } else { cpu.regs.x(rn) }
}

fn apply_extension(val: u64, option: u8) -> u64 {
    match option {
        0b010 => (val as u32) as u64,
        0b110 => (val as i32) as i64 as u64,
        0b011 => val,
        0b111 => val,
        _ => val,
    }
}

fn ldst_size(instr: &Instr) -> u8 {
    if instr.size != 0 { instr.size } else if instr.sf { 8 } else { 4 }
}

pub(super) fn exec_ldr_str(cpu: &mut Armv8Cpu, bus: &mut SystemBus, instr: Instr) -> Result<(), &'static str> {
    let (va, writeback) = compute_ldst_va(cpu, &instr);
    let size = ldst_size(&instr);
    let is_load = instr.op == Opcode::Ldr;

    let pa = translate(&cpu.sys, &mut cpu.tlb, &bus.mem, va).map_err(|_| "LDR/STR translation fault")?;
    if is_load {
        let val = bus.read(pa, size).ok_or("LDR bus fault")?;
        write_reg(cpu, instr.rd, val, instr.sf);
    } else {
        let val = read_reg(cpu, instr.rd, instr.sf);
        bus.write(pa, size, val);
    }
    if let Some(new_base) = writeback {
        write_reg_sp(cpu, instr.rn, new_base, true);
    }
    Ok(())
}

pub(super) fn exec_ldr_lit(cpu: &mut Armv8Cpu, bus: &mut SystemBus, instr: Instr) -> Result<(), &'static str> {
    let va = branch_target(cpu.regs.pc, instr.imm);
    let size = if instr.sf { 8 } else { 4 };
    let pa = translate(&cpu.sys, &mut cpu.tlb, &bus.mem, va).map_err(|_| "LDR literal translation fault")?;
    let val = bus.read(pa, size).ok_or("LDR literal bus fault")?;
    write_reg(cpu, instr.rd, val, instr.sf);
    Ok(())
}

pub(super) fn exec_ldp_stp(cpu: &mut Armv8Cpu, bus: &mut SystemBus, instr: Instr) -> Result<(), &'static str> {
    let base = read_base(cpu, instr.rn, true);
    let size = if instr.size != 0 { instr.size as u64 } else if instr.sf { 8u64 } else { 4u64 };
    let (va, new_base) = match instr.cond {
        1 => (base, branch_target(base, instr.imm)),
        3 => { let b = branch_target(base, instr.imm); (b, b) },
        _ => (branch_target(base, instr.imm), base),
    };
    let pa1 = translate(&cpu.sys, &mut cpu.tlb, &bus.mem, va).map_err(|_| "LDP/STP translation fault")?;
    let pa2 = translate(&cpu.sys, &mut cpu.tlb, &bus.mem, va + size).map_err(|_| "LDP/STP translation fault")?;

    match instr.op {
        Opcode::Ldp  => { write_reg(cpu, instr.rd, bus.read(pa1, size as u8).ok_or("LDP bus fault")?, instr.sf); write_reg(cpu, instr.rm, bus.read(pa2, size as u8).ok_or("LDP bus fault")?, instr.sf); }
        Opcode::Stp  => { bus.write(pa1, size as u8, read_reg(cpu, instr.rd, instr.sf)); bus.write(pa2, size as u8, read_reg(cpu, instr.rm, instr.sf)); }
        Opcode::SimdLdp => { let _ = bus.read(pa1, size as u8); let _ = bus.read(pa2, size as u8); }
        Opcode::SimdStp => { bus.write(pa1, size as u8, 0); bus.write(pa2, size as u8, 0); }
        _ => unreachable!(),
    }
    if new_base != base { write_reg_sp(cpu, instr.rn, new_base, true); }
    Ok(())
}

pub(super) fn exec_exclusive(cpu: &mut Armv8Cpu, bus: &mut SystemBus, instr: Instr) -> Result<(), &'static str> {
    let base = base_addr(cpu, instr.rn);
    match instr.op {
        Opcode::Ldxr | Opcode::Ldar => {
            let pa = translate(&cpu.sys, &mut cpu.tlb, &bus.mem, base).map_err(|_| "LDXR translation fault")?;
            let val = bus.read(pa, instr.size).ok_or("LDXR bus fault")?;
            write_reg(cpu, instr.rd, val, instr.sf);
        }
        Opcode::Stxr | Opcode::Stlr => {
            let pa = translate(&cpu.sys, &mut cpu.tlb, &bus.mem, base).map_err(|_| "STXR translation fault")?;
            bus.write(pa, instr.size, read_reg(cpu, instr.rd, instr.sf));
            if instr.op == Opcode::Stxr {
                write_reg(cpu, instr.imm as u8, 0, false);
            }
        }
        Opcode::Ldxp => {
            let size = if instr.sf { 8 } else { 4 };
            let pa1 = translate(&cpu.sys, &mut cpu.tlb, &bus.mem, base).map_err(|_| "LDXP fault")?;
            let pa2 = translate(&cpu.sys, &mut cpu.tlb, &bus.mem, base + size).map_err(|_| "LDXP fault")?;
            write_reg(cpu, instr.rd, bus.read(pa1, size as u8).ok_or("LDXP fault")?, instr.sf);
            write_reg(cpu, instr.rm, bus.read(pa2, size as u8).ok_or("LDXP fault")?, instr.sf);
        }
        Opcode::Stxp => {
            let size = if instr.sf { 8 } else { 4 };
            let pa1 = translate(&cpu.sys, &mut cpu.tlb, &bus.mem, base).map_err(|_| "STXP fault")?;
            let pa2 = translate(&cpu.sys, &mut cpu.tlb, &bus.mem, base + size).map_err(|_| "STXP fault")?;
            bus.write(pa1, size as u8, read_reg(cpu, instr.rd, instr.sf));
            bus.write(pa2, size as u8, read_reg(cpu, instr.rm, instr.sf));
            write_reg(cpu, instr.imm as u8, 0, false);
        }
        _ => unreachable!(),
    }
    Ok(())
}
