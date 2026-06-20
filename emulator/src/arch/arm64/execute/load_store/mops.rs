use super::*;

pub(in crate::arch::arm64::execute) fn exec_mops(
    cpu: &mut Armv8Cpu,
    bus: &mut SystemBus,
    instr: Instr,
) -> Result<(), &'static str> {
    if instr.op == Opcode::MopsSetP || instr.op == Opcode::MopsSetM || instr.op == Opcode::MopsSetE
    {
        return exec_set(cpu, bus, instr);
    }
    exec_copy(cpu, bus, instr)
}

fn exec_copy(cpu: &mut Armv8Cpu, bus: &mut SystemBus, instr: Instr) -> Result<(), &'static str> {
    let mut dst = read_reg(cpu, instr.rd, true);
    let mut src = read_reg(cpu, instr.rm, true);
    let size = copy_size(cpu, instr);
    let backward = copy_direction_backward(cpu, instr, dst, src, size);
    let stage_start = if backward && !is_prologue(instr.op) {
        dst = dst.wrapping_sub(size);
        src = src.wrapping_sub(size);
        (dst, src)
    } else {
        (dst, src)
    };

    if backward {
        copy_backward_bytes(cpu, bus, stage_start.0, stage_start.1, size)?;
    } else {
        copy_forward_bytes(cpu, bus, stage_start.0, stage_start.1, size)?;
        dst = dst.wrapping_add(size);
        src = src.wrapping_add(size);
    }

    write_reg(cpu, instr.rn, 0, true);
    write_reg(cpu, instr.rd, dst, true);
    write_reg(cpu, instr.rm, src, true);
    if is_prologue(instr.op) {
        cpu.pstate.set_nzcv(backward, false, true, false);
    }
    Ok(())
}

fn exec_set(cpu: &mut Armv8Cpu, bus: &mut SystemBus, instr: Instr) -> Result<(), &'static str> {
    let dst = read_reg(cpu, instr.rd, true);
    let size = set_size(cpu, instr);
    let byte = read_reg(cpu, instr.rm, true) & 0xFF;

    for offset in 0..size {
        write_guest(
            cpu,
            bus,
            dst.wrapping_add(offset),
            1,
            byte,
            "MOPS SET bus fault",
        )?;
    }

    write_reg(cpu, instr.rn, 0, true);
    write_reg(cpu, instr.rd, dst.wrapping_add(size), true);
    if instr.op == Opcode::MopsSetP {
        cpu.pstate.set_nzcv(false, false, true, false);
    }
    Ok(())
}

fn copy_forward_bytes(
    cpu: &mut Armv8Cpu,
    bus: &mut SystemBus,
    dst: u64,
    src: u64,
    size: u64,
) -> Result<(), &'static str> {
    for offset in 0..size {
        let byte = read_guest(cpu, bus, src.wrapping_add(offset), 1, "MOPS CPY bus fault")?;
        write_guest(
            cpu,
            bus,
            dst.wrapping_add(offset),
            1,
            byte,
            "MOPS CPY bus fault",
        )?;
    }
    Ok(())
}

fn copy_backward_bytes(
    cpu: &mut Armv8Cpu,
    bus: &mut SystemBus,
    dst: u64,
    src: u64,
    size: u64,
) -> Result<(), &'static str> {
    for offset in (0..size).rev() {
        let byte = read_guest(cpu, bus, src.wrapping_add(offset), 1, "MOPS CPY bus fault")?;
        write_guest(
            cpu,
            bus,
            dst.wrapping_add(offset),
            1,
            byte,
            "MOPS CPY bus fault",
        )?;
    }
    Ok(())
}

fn copy_size(cpu: &Armv8Cpu, instr: Instr) -> u64 {
    let raw = read_reg(cpu, instr.rn, true);
    if is_prologue(instr.op) && raw >> 63 != 0 {
        0x7FFF_FFFF_FFFF_FFFF
    } else if !is_prologue(instr.op) && !cpu.pstate.c() && raw >> 63 != 0 {
        raw.wrapping_neg()
    } else {
        raw
    }
}

fn set_size(cpu: &Armv8Cpu, instr: Instr) -> u64 {
    let raw = read_reg(cpu, instr.rn, true);
    if instr.op == Opcode::MopsSetP && raw >> 63 != 0 {
        0x7FFF_FFFF_FFFF_FFFF
    } else if instr.op != Opcode::MopsSetP && !cpu.pstate.c() && raw >> 63 != 0 {
        raw.wrapping_neg()
    } else {
        raw
    }
}

fn copy_direction_backward(cpu: &Armv8Cpu, instr: Instr, dst: u64, src: u64, size: u64) -> bool {
    if instr.op == Opcode::MopsCpyP {
        return copy_backward(dst, src, size);
    }
    (instr.op == Opcode::MopsCpyM || instr.op == Opcode::MopsCpyE) && cpu.pstate.n()
}

fn copy_backward(dst: u64, src: u64, size: u64) -> bool {
    let d = dst & 0x00FF_FFFF_FFFF_FFFF;
    let s = src & 0x00FF_FFFF_FFFF_FFFF;
    s < d && s.wrapping_add(size) > d
}

fn is_prologue(op: Opcode) -> bool {
    op == Opcode::MopsCpyFp || op == Opcode::MopsCpyP || op == Opcode::MopsSetP
}
