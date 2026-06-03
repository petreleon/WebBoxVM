//! Load/Store instruction execution.

use super::{Instr, Opcode, branch_target};
use crate::arm64::Armv8Cpu;
use crate::arm64::helpers::{read_base, read_reg, write_reg, write_reg_sp};
use crate::arm64::mmu::{Fault, translate, translate_write};
use crate::bus::SystemBus;
use crate::constants::*;
use std::env;

const SIMD_MULTI_POST_INDEX: u8 = 0xFE;

fn compute_ldst_va(cpu: &Armv8Cpu, instr: &Instr) -> (u64, Option<u64>) {
    if instr.rm == SIMD_MULTI_POST_INDEX {
        let base = base_addr(cpu, instr.rn);
        (base, Some(base.wrapping_add(instr.imm)))
    } else if instr.rm != 0xFF {
        let base = base_addr(cpu, instr.rn);
        let offset_val = read_reg(cpu, instr.rm, true);
        let extended = apply_extension(offset_val, instr.cond);
        let shift = if instr.imm == 1 {
            instr.size.trailing_zeros() as u8
        } else {
            0
        };
        (base.wrapping_add(extended << shift), None)
    } else {
        let base = base_addr(cpu, instr.rn);
        let (va, wb) = match instr.cond {
            1 => (base, Some(base.wrapping_add(instr.imm))),
            3 => {
                let b = base.wrapping_add(instr.imm);
                (b, Some(b))
            }
            _ => (base.wrapping_add(instr.imm), None),
        };
        (va, wb)
    }
}

fn base_addr(cpu: &Armv8Cpu, rn: u8) -> u64 {
    if rn == SP_REGISTER_INDEX {
        cpu.regs.sp
    } else {
        cpu.regs.x(rn)
    }
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
    if instr.size != 0 {
        instr.size
    } else if instr.sf {
        8
    } else {
        4
    }
}

pub(super) fn exec_ldr_str(
    cpu: &mut Armv8Cpu,
    bus: &mut SystemBus,
    instr: Instr,
) -> Result<(), &'static str> {
    let (va, writeback) = compute_ldst_va(cpu, &instr);
    let size = ldst_size(&instr);
    let is_load = matches!(
        instr.op,
        Opcode::Ldr
            | Opcode::LdrSign
            | Opcode::SimdLdr
            | Opcode::SimdLd1
            | Opcode::SimdLd1Multi
            | Opcode::SimdLd1Lane
            | Opcode::SimdLd1r
            | Opcode::SimdLd4
    );

    let pa = translate_or_data_fault(cpu, &mut bus.mem, va, !is_load, "LDR/STR translation fault")?;
    if instr.op == Opcode::SimdLdr {
        cpu.simd[instr.rd as usize] = read_simd_guest(cpu, bus, va, size, "SIMD load fault")?;
    } else if instr.op == Opcode::SimdLd1 {
        cpu.simd[instr.rd as usize] = read_simd_guest(cpu, bus, va, size, "SIMD load fault")?;
    } else if instr.op == Opcode::SimdLd1Multi {
        exec_ld1_multi(cpu, bus, va, instr)?;
    } else if instr.op == Opcode::SimdLd1Lane {
        let lane = instr.imm as u32;
        let mask = !((u64::MAX as u128) << (lane * 64));
        let value = read_guest(cpu, bus, va, 8, "SIMD lane load fault")? as u128;
        cpu.simd[instr.rd as usize] = (cpu.simd[instr.rd as usize] & mask) | (value << (lane * 64));
    } else if instr.op == Opcode::SimdLd1r {
        let element_size = instr.cond.max(1);
        let value = read_guest(cpu, bus, va, element_size, "LD1R bus fault")? as u128;
        cpu.simd[instr.rd as usize] =
            super::alu::simd_replicate_element(value, element_size as usize, instr.size as usize);
    } else if instr.op == Opcode::SimdLd4 {
        exec_ld4(cpu, bus, va, instr)?;
    } else if matches!(instr.op, Opcode::SimdStr | Opcode::SimdSt4Single) {
        write_simd_guest(
            cpu,
            bus,
            va,
            size,
            cpu.simd[instr.rd as usize],
            "SIMD store fault",
        )?;
    } else if instr.op == Opcode::SimdSt1Multi {
        exec_st1_multi(cpu, bus, va, instr)?;
    } else if instr.op == Opcode::SimdSt4 {
        exec_st4(cpu, bus, va, instr)?;
    } else if is_load {
        let mut val = read_guest(cpu, bus, va, size, "LDR bus fault")?;
        trace_syscall_frame_access(cpu, &instr, "LDR", va, pa, size, Some(val));
        if instr.op == Opcode::LdrSign {
            val = sign_extend_load(val, size, instr.sf);
        }
        write_reg(cpu, instr.rd, val, instr.sf);
    } else {
        let val = read_reg(cpu, instr.rd, instr.sf);
        trace_syscall_frame_access(cpu, &instr, "STR", va, pa, size, Some(val));
        trace_text_store(cpu, bus, &instr, "STR", va, pa, size, val);
        write_guest(cpu, bus, va, size, val, "STR translation fault")?;
    }
    if let Some(new_base) = writeback {
        write_reg_sp(cpu, instr.rn, new_base, true);
    }
    Ok(())
}

fn exec_ld1_multi(
    cpu: &mut Armv8Cpu,
    bus: &mut SystemBus,
    va: u64,
    instr: Instr,
) -> Result<(), &'static str> {
    let register_count = instr.cond.max(1) as usize;
    let vector_size = ldst_size(&instr) as u64;
    for register_index in 0..register_count {
        let reg = ((instr.rd as usize) + register_index) & 31;
        let reg_va = va.wrapping_add(register_index as u64 * vector_size);
        cpu.simd[reg] = read_simd_guest(cpu, bus, reg_va, instr.size, "LD1 multi bus fault")?;
    }
    Ok(())
}

fn exec_st1_multi(
    cpu: &mut Armv8Cpu,
    bus: &mut SystemBus,
    va: u64,
    instr: Instr,
) -> Result<(), &'static str> {
    let register_count = instr.cond.max(1) as usize;
    let vector_size = ldst_size(&instr) as u64;
    for register_index in 0..register_count {
        let reg = ((instr.rd as usize) + register_index) & 31;
        let reg_va = va.wrapping_add(register_index as u64 * vector_size);
        write_simd_guest(
            cpu,
            bus,
            reg_va,
            instr.size,
            cpu.simd[reg],
            "ST1 multi bus fault",
        )?;
    }
    Ok(())
}

fn exec_ld4(
    cpu: &mut Armv8Cpu,
    bus: &mut SystemBus,
    va: u64,
    instr: Instr,
) -> Result<(), &'static str> {
    let lanes = simd_structure_lanes(instr)?;
    let element_size = 1usize << instr.cond;
    let mut regs = [0u128; 4];
    for lane in 0..lanes {
        for reg_index in 0..4 {
            let mut element = 0u128;
            for byte_index in 0..element_size {
                let byte_offset = ((lane * 4 + reg_index) * element_size + byte_index) as u64;
                let pa = translate_or_data_fault(
                    cpu,
                    &mut bus.mem,
                    va.wrapping_add(byte_offset),
                    false,
                    "LD4 translation fault",
                )?;
                let byte = bus.read(pa, 1).ok_or("LD4 bus fault")? as u128;
                element |= byte << (byte_index * 8);
            }
            regs[reg_index] |= element << (lane * element_size * 8);
        }
    }
    for (offset, value) in regs.into_iter().enumerate() {
        cpu.simd[((instr.rd as usize) + offset) & 31] = value;
    }
    Ok(())
}

fn exec_st4(
    cpu: &mut Armv8Cpu,
    bus: &mut SystemBus,
    va: u64,
    instr: Instr,
) -> Result<(), &'static str> {
    let lanes = simd_structure_lanes(instr)?;
    let element_size = 1usize << instr.cond;
    for lane in 0..lanes {
        for reg_index in 0..4 {
            let value = cpu.simd[((instr.rd as usize) + reg_index) & 31];
            for byte_index in 0..element_size {
                let byte_offset = ((lane * 4 + reg_index) * element_size + byte_index) as u64;
                let pa = translate_or_data_fault(
                    cpu,
                    &mut bus.mem,
                    va.wrapping_add(byte_offset),
                    true,
                    "ST4 translation fault",
                )?;
                let byte = (value >> (lane * element_size * 8 + byte_index * 8)) & 0xff;
                bus.write(pa, 1, byte as u64);
                cpu.clear_exclusive_if_overlaps(pa, 1);
            }
        }
    }
    Ok(())
}

fn simd_structure_lanes(instr: Instr) -> Result<usize, &'static str> {
    let element_size = 1usize << instr.cond;
    if !matches!(element_size, 1 | 2 | 4 | 8) {
        return Err("unsupported SIMD structure element size");
    }
    Ok((instr.size as usize) / element_size)
}

fn sign_extend_load(val: u64, size: u8, sf: bool) -> u64 {
    match (size, sf) {
        (1, false) => (val as i8 as i32) as u32 as u64,
        (1, true) => val as i8 as i64 as u64,
        (2, false) => (val as i16 as i32) as u32 as u64,
        (2, true) => val as i16 as i64 as u64,
        (4, true) => val as u32 as i32 as i64 as u64,
        _ => val,
    }
}

fn read_simd_guest(
    cpu: &mut Armv8Cpu,
    bus: &mut SystemBus,
    va: u64,
    size: u8,
    err: &'static str,
) -> Result<u128, &'static str> {
    match size {
        1 => Ok(read_guest(cpu, bus, va, 1, err)? as u8 as u128),
        2 => Ok(read_guest(cpu, bus, va, 2, err)? as u16 as u128),
        4 => Ok(read_guest(cpu, bus, va, 4, err)? as u32 as u128),
        8 => Ok(read_guest(cpu, bus, va, 8, err)? as u128),
        16 => {
            let lo = read_guest(cpu, bus, va, 8, err)? as u128;
            let hi = read_guest(cpu, bus, va.wrapping_add(8), 8, err)? as u128;
            Ok(lo | (hi << 64))
        }
        _ => Err("unsupported SIMD load size"),
    }
}

fn write_simd_guest(
    cpu: &mut Armv8Cpu,
    bus: &mut SystemBus,
    va: u64,
    size: u8,
    value: u128,
    err: &'static str,
) -> Result<(), &'static str> {
    match size {
        1 => write_guest(cpu, bus, va, 1, value as u8 as u64, err)?,
        2 => write_guest(cpu, bus, va, 2, value as u16 as u64, err)?,
        4 => write_guest(cpu, bus, va, 4, value as u32 as u64, err)?,
        8 => write_guest(cpu, bus, va, 8, value as u64, err)?,
        16 => {
            write_guest(cpu, bus, va, 8, value as u64, err)?;
            write_guest(cpu, bus, va.wrapping_add(8), 8, (value >> 64) as u64, err)?;
        }
        _ => return Err("unsupported SIMD store size"),
    }
    Ok(())
}

fn read_guest(
    cpu: &mut Armv8Cpu,
    bus: &mut SystemBus,
    va: u64,
    size: u8,
    err: &'static str,
) -> Result<u64, &'static str> {
    if !access_crosses_page(va, size) {
        let pa = translate_or_data_fault(cpu, &mut bus.mem, va, false, err)?;
        return bus.read(pa, size).ok_or(err);
    }

    let mut value = 0u64;
    for offset in 0..size {
        let pa = translate_or_data_fault(
            cpu,
            &mut bus.mem,
            va.wrapping_add(offset as u64),
            false,
            err,
        )?;
        let byte = bus.read(pa, 1).ok_or(err)?;
        value |= byte << (offset * 8);
    }
    Ok(value)
}

fn write_guest(
    cpu: &mut Armv8Cpu,
    bus: &mut SystemBus,
    va: u64,
    size: u8,
    value: u64,
    err: &'static str,
) -> Result<(), &'static str> {
    if !access_crosses_page(va, size) {
        let pa = translate_or_data_fault(cpu, &mut bus.mem, va, true, err)?;
        bus.write(pa, size, value);
        cpu.clear_exclusive_if_overlaps(pa, size);
        return Ok(());
    }

    for offset in 0..size {
        let byte_va = va.wrapping_add(offset as u64);
        let pa = translate_or_data_fault(cpu, &mut bus.mem, byte_va, true, err)?;
        bus.write(pa, 1, (value >> (offset * 8)) & 0xff);
        cpu.clear_exclusive_if_overlaps(pa, 1);
    }
    Ok(())
}

fn access_crosses_page(va: u64, size: u8) -> bool {
    (va & PAGE_OFFSET_MASK) + size as u64 > PAGE_SIZE
}

pub(super) fn exec_ldr_lit(
    cpu: &mut Armv8Cpu,
    bus: &mut SystemBus,
    instr: Instr,
) -> Result<(), &'static str> {
    let va = branch_target(cpu.regs.pc, instr.imm);
    let size = if instr.sf { 8 } else { 4 };
    let val = read_guest(cpu, bus, va, size, "LDR literal bus fault")?;
    write_reg(cpu, instr.rd, val, instr.sf);
    Ok(())
}

pub(super) fn exec_ldp_stp(
    cpu: &mut Armv8Cpu,
    bus: &mut SystemBus,
    instr: Instr,
) -> Result<(), &'static str> {
    let base = read_base(cpu, instr.rn, true);
    let size = if instr.size != 0 {
        instr.size as u64
    } else if instr.sf {
        8u64
    } else {
        4u64
    };
    let (va, new_base) = match instr.cond {
        1 => (base, branch_target(base, instr.imm)),
        3 => {
            let b = branch_target(base, instr.imm);
            (b, b)
        }
        _ => (branch_target(base, instr.imm), base),
    };
    let is_store = matches!(instr.op, Opcode::Stp | Opcode::SimdStp);
    let pa1 =
        translate_or_data_fault(cpu, &mut bus.mem, va, is_store, "LDP/STP translation fault")?;
    let pa2 = translate_or_data_fault(
        cpu,
        &mut bus.mem,
        va + size,
        is_store,
        "LDP/STP translation fault",
    )?;

    match instr.op {
        Opcode::Ldp => {
            let lo = read_guest(cpu, bus, va, size as u8, "LDP bus fault")?;
            let hi = read_guest(cpu, bus, va + size, size as u8, "LDP bus fault")?;
            trace_syscall_frame_access(cpu, &instr, "LDP.0", va, pa1, size as u8, Some(lo));
            trace_syscall_frame_access(cpu, &instr, "LDP.1", va + size, pa2, size as u8, Some(hi));
            write_reg(cpu, instr.rd, lo, instr.sf);
            write_reg(cpu, instr.rm, hi, instr.sf);
        }
        Opcode::Ldpsw => {
            let lo = read_guest(cpu, bus, va, 4, "LDPSW bus fault")? as u32 as i32 as i64 as u64;
            let hi =
                read_guest(cpu, bus, va + 4, 4, "LDPSW bus fault")? as u32 as i32 as i64 as u64;
            trace_syscall_frame_access(cpu, &instr, "LDPSW.0", va, pa1, 4, Some(lo));
            trace_syscall_frame_access(cpu, &instr, "LDPSW.1", va + 4, pa2, 4, Some(hi));
            write_reg(cpu, instr.rd, lo, true);
            write_reg(cpu, instr.rm, hi, true);
        }
        Opcode::Stp => {
            let access_size = size as u8;
            let val1 = read_reg(cpu, instr.rd, instr.sf);
            let val2 = read_reg(cpu, instr.rm, instr.sf);
            trace_syscall_frame_access(cpu, &instr, "STP.0", va, pa1, access_size, Some(val1));
            trace_syscall_frame_access(
                cpu,
                &instr,
                "STP.1",
                va + size,
                pa2,
                access_size,
                Some(val2),
            );
            trace_text_store(cpu, bus, &instr, "STP.0", va, pa1, access_size, val1);
            write_guest(cpu, bus, va, access_size, val1, "STP bus fault")?;
            trace_text_store(cpu, bus, &instr, "STP.1", va + size, pa2, access_size, val2);
            write_guest(cpu, bus, va + size, access_size, val2, "STP bus fault")?;
        }
        Opcode::SimdLdp => {
            let access_size = size as u8;
            cpu.simd[instr.rd as usize] =
                read_simd_guest(cpu, bus, va, access_size, "SIMD LDP bus fault")?;
            cpu.simd[instr.rm as usize] =
                read_simd_guest(cpu, bus, va + size, access_size, "SIMD LDP bus fault")?;
        }
        Opcode::SimdStp => {
            let access_size = size as u8;
            write_simd_guest(
                cpu,
                bus,
                va,
                access_size,
                cpu.simd[instr.rd as usize],
                "SIMD STP bus fault",
            )?;
            write_simd_guest(
                cpu,
                bus,
                va + size,
                access_size,
                cpu.simd[instr.rm as usize],
                "SIMD STP bus fault",
            )?;
        }
        _ => unreachable!(),
    }
    if new_base != base {
        write_reg_sp(cpu, instr.rn, new_base, true);
    }
    Ok(())
}

fn trace_syscall_frame_access(
    cpu: &mut Armv8Cpu,
    instr: &Instr,
    kind: &str,
    va: u64,
    pa: u64,
    size: u8,
    value: Option<u64>,
) {
    if env::var_os("WEBBOXVM_TRACE_SYSCALL_FRAME").is_none()
        || cpu.pstate.el() != 1
        || cpu.trace_syscall_access_budget == 0
        || cpu.trace_syscall_stack_top == 0
    {
        return;
    }

    let stack_top = cpu.trace_syscall_stack_top;
    if !(stack_top.saturating_sub(0x300)..=stack_top).contains(&va) {
        return;
    }
    cpu.trace_syscall_access_budget -= 1;

    eprintln!(
        "FRAME {kind} pc={:#018x} va={:#018x} top_off=-{:#x} sp_off={:#x} pa={:#018x} size={} rd={} rn={} rm={} imm={:#x} base={:#018x} value={:#018x}",
        cpu.regs.pc,
        va,
        stack_top.wrapping_sub(va),
        va.wrapping_sub(cpu.regs.sp),
        pa,
        size,
        instr.rd,
        instr.rn,
        instr.rm,
        instr.imm,
        base_addr(cpu, instr.rn),
        value.unwrap_or(0),
    );
}

pub(super) fn exec_exclusive(
    cpu: &mut Armv8Cpu,
    bus: &mut SystemBus,
    instr: Instr,
) -> Result<(), &'static str> {
    let base = base_addr(cpu, instr.rn);
    match instr.op {
        Opcode::Ldxr => {
            let pa =
                translate_or_data_fault(cpu, &mut bus.mem, base, false, "LDXR translation fault")?;
            let val = bus.read(pa, instr.size).ok_or("LDXR bus fault")?;
            cpu.reserve_exclusive(pa, instr.size);
            write_reg(cpu, instr.rd, val, instr.sf);
        }
        Opcode::Ldar => {
            let pa =
                translate_or_data_fault(cpu, &mut bus.mem, base, false, "LDAR translation fault")?;
            let val = bus.read(pa, instr.size).ok_or("LDAR bus fault")?;
            write_reg(cpu, instr.rd, val, instr.sf);
        }
        Opcode::Stxr => {
            let pa =
                translate_or_data_fault(cpu, &mut bus.mem, base, true, "STXR translation fault")?;
            let success = cpu.exclusive_matches(pa, instr.size);
            if success {
                let val = read_reg(cpu, instr.rd, instr.sf);
                trace_text_store(cpu, bus, &instr, "STXR", base, pa, instr.size, val);
                bus.write(pa, instr.size, val);
            }
            cpu.clear_exclusive();
            write_reg(cpu, instr.imm as u8, if success { 0 } else { 1 }, false);
        }
        Opcode::Stlr => {
            let pa =
                translate_or_data_fault(cpu, &mut bus.mem, base, true, "STLR translation fault")?;
            let val = read_reg(cpu, instr.rd, instr.sf);
            trace_text_store(cpu, bus, &instr, "STLR", base, pa, instr.size, val);
            bus.write(pa, instr.size, val);
            cpu.clear_exclusive_if_overlaps(pa, instr.size);
        }
        Opcode::Ldxp => {
            let size = if instr.sf { 8 } else { 4 };
            let pa1 =
                translate_or_data_fault(cpu, &mut bus.mem, base, false, "LDXP translation fault")?;
            let pa2 = translate_or_data_fault(
                cpu,
                &mut bus.mem,
                base + size,
                false,
                "LDXP translation fault",
            )?;
            write_reg(
                cpu,
                instr.rd,
                bus.read(pa1, size as u8).ok_or("LDXP fault")?,
                instr.sf,
            );
            write_reg(
                cpu,
                instr.rm,
                bus.read(pa2, size as u8).ok_or("LDXP fault")?,
                instr.sf,
            );
            cpu.reserve_exclusive(pa1, (size * 2) as u8);
        }
        Opcode::Stxp => {
            let size = if instr.sf { 8 } else { 4 };
            let pa1 =
                translate_or_data_fault(cpu, &mut bus.mem, base, true, "STXP translation fault")?;
            let pa2 = translate_or_data_fault(
                cpu,
                &mut bus.mem,
                base + size,
                true,
                "STXP translation fault",
            )?;
            let total_size = (size * 2) as u8;
            let success = cpu.exclusive_matches(pa1, total_size);
            if success {
                let val1 = read_reg(cpu, instr.rd, instr.sf);
                let val2 = read_reg(cpu, instr.rm, instr.sf);
                trace_text_store(cpu, bus, &instr, "STXP.0", base, pa1, size as u8, val1);
                bus.write(pa1, size as u8, val1);
                trace_text_store(
                    cpu,
                    bus,
                    &instr,
                    "STXP.1",
                    base + size,
                    pa2,
                    size as u8,
                    val2,
                );
                bus.write(pa2, size as u8, val2);
            }
            cpu.clear_exclusive();
            write_reg(cpu, instr.imm as u8, if success { 0 } else { 1 }, false);
        }
        _ => unreachable!(),
    }
    Ok(())
}

pub(super) fn exec_atomic(
    cpu: &mut Armv8Cpu,
    bus: &mut SystemBus,
    instr: Instr,
) -> Result<(), &'static str> {
    let base = base_addr(cpu, instr.rn);
    let pa = translate_or_data_fault(cpu, &mut bus.mem, base, true, "atomic translation fault")?;

    match instr.op {
        Opcode::Atomic => {
            let size = instr.size;
            let old = bus.read(pa, size).ok_or("atomic bus fault")?;
            let source = read_reg(cpu, instr.rm, instr.sf) & access_mask(size);
            let new = atomic_result(instr.imm as u8, old, source, size)?;
            trace_text_store(cpu, bus, &instr, "ATOMIC", base, pa, size, new);
            bus.write(pa, size, new);
            cpu.clear_exclusive_if_overlaps(pa, size);
            write_reg(cpu, instr.rd, old, instr.sf);
        }
        Opcode::Cas => {
            let size = instr.size;
            let mask = access_mask(size);
            let old = bus.read(pa, size).ok_or("CAS bus fault")?;
            let expected = read_reg(cpu, instr.rd, instr.sf) & mask;
            if old == expected {
                let val = read_reg(cpu, instr.rm, instr.sf) & mask;
                trace_text_store(cpu, bus, &instr, "CAS", base, pa, size, val);
                bus.write(pa, size, val);
                cpu.clear_exclusive_if_overlaps(pa, size);
            }
            write_reg(cpu, instr.rd, old, instr.sf);
        }
        Opcode::Casp => {
            let size = instr.size;
            let mask = access_mask(size);
            let old_lo = bus.read(pa, size).ok_or("CASP bus fault")?;
            let old_hi = bus.read(pa + size as u64, size).ok_or("CASP bus fault")?;
            let expected_lo = read_reg(cpu, instr.rd, instr.sf) & mask;
            let expected_hi = read_reg(cpu, instr.rd + 1, instr.sf) & mask;
            if old_lo == expected_lo && old_hi == expected_hi {
                let val1 = read_reg(cpu, instr.rm, instr.sf) & mask;
                let val2 = read_reg(cpu, instr.rm + 1, instr.sf) & mask;
                trace_text_store(cpu, bus, &instr, "CASP.0", base, pa, size, val1);
                bus.write(pa, size, val1);
                cpu.clear_exclusive_if_overlaps(pa, size);
                trace_text_store(
                    cpu,
                    bus,
                    &instr,
                    "CASP.1",
                    base + size as u64,
                    pa + size as u64,
                    size,
                    val2,
                );
                bus.write(pa + size as u64, size, val2);
                cpu.clear_exclusive_if_overlaps(pa + size as u64, size);
            }
            write_reg(cpu, instr.rd, old_lo, instr.sf);
            write_reg(cpu, instr.rd + 1, old_hi, instr.sf);
        }
        _ => unreachable!(),
    }

    Ok(())
}

fn trace_text_store(
    cpu: &Armv8Cpu,
    bus: &mut SystemBus,
    instr: &Instr,
    kind: &str,
    va: u64,
    pa: u64,
    size: u8,
    value: u64,
) {
    let trace_text = env::var_os("WEBBOXVM_TRACE_TEXT_PATCH").is_some();
    let trace_store = env::var_os("WEBBOXVM_TRACE_STORE_PA").and_then(|target| {
        let target = target.to_string_lossy();
        u64::from_str_radix(target.trim_start_matches("0x"), 16).ok()
    });
    if !trace_text && trace_store.is_none() {
        return;
    }
    let start = pa;
    let end = pa.saturating_add(size as u64);
    if let Some(target) = trace_store {
        if target < start || target >= end {
            return;
        }
    } else if end <= 0x4003_6e40 || start >= 0x4003_6ec8 {
        return;
    }
    let old = bus.read(pa, size).unwrap_or(0);
    eprintln!(
        "TEXT STORE {kind} pc=0x{:016x} instr={instr:?} va=0x{va:016x} pa=0x{pa:016x} size={size} old=0x{old:016x} new=0x{value:016x} \
         x0=0x{:016x} x1=0x{:016x} x2=0x{:016x} x3=0x{:016x} x4=0x{:016x} x5=0x{:016x} \
         x19=0x{:016x} x20=0x{:016x} x21=0x{:016x} lr=0x{:016x} sp=0x{:016x}",
        cpu.regs.pc,
        cpu.regs.x(0),
        cpu.regs.x(1),
        cpu.regs.x(2),
        cpu.regs.x(3),
        cpu.regs.x(4),
        cpu.regs.x(5),
        cpu.regs.x(19),
        cpu.regs.x(20),
        cpu.regs.x(21),
        cpu.regs.x(30),
        cpu.regs.sp,
    );
}

fn atomic_result(op: u8, old: u64, source: u64, size: u8) -> Result<u64, &'static str> {
    let mask = access_mask(size);
    let result = match op & 0xF {
        0x0 => old.wrapping_add(source),
        0x1 => old & !source,
        0x2 => old ^ source,
        0x3 => old | source,
        0x4 => signed_ext(old, size).max(signed_ext(source, size)) as u64,
        0x5 => signed_ext(old, size).min(signed_ext(source, size)) as u64,
        0x6 => old.max(source),
        0x7 => old.min(source),
        0x8 => source,
        _ => return Err("unsupported atomic operation"),
    };
    Ok(result & mask)
}

fn access_mask(size: u8) -> u64 {
    match size {
        1 => 0xFF,
        2 => 0xFFFF,
        4 => 0xFFFF_FFFF,
        8 => u64::MAX,
        _ => 0,
    }
}

fn signed_ext(val: u64, size: u8) -> i64 {
    match size {
        1 => val as i8 as i64,
        2 => val as i16 as i64,
        4 => val as u32 as i32 as i64,
        8 => val as i64,
        _ => val as i64,
    }
}

fn translate_or_data_fault(
    cpu: &mut Armv8Cpu,
    mem: &mut crate::memory::PhysicalMemory,
    va: u64,
    write: bool,
    err: &'static str,
) -> Result<u64, &'static str> {
    let result = if write {
        translate_write(&cpu.sys, mem, va, cpu.pstate.el())
    } else {
        translate(&cpu.sys, &mut cpu.tlb, mem, va)
    };

    match result {
        Ok(pa) => Ok(pa),
        Err(
            fault @ (Fault::TranslationFault | Fault::AccessFlagFault | Fault::PermissionFault),
        ) => {
            cpu.sys.far_el1 = va;
            Err(match fault {
                Fault::TranslationFault => err,
                Fault::AccessFlagFault => "access flag fault",
                Fault::PermissionFault => "permission fault",
            })
        }
    }
}
