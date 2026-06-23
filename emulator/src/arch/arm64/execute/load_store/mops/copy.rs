use super::*;

const MOPS_PAGE_BYTES: usize = PAGE_SIZE as usize;

pub(super) fn exec_copy(
    cpu: &mut Armv8Cpu,
    bus: &mut SystemBus,
    instr: Instr,
) -> Result<(), &'static str> {
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

    if !try_bulk_copy_same_page(cpu, bus, stage_start.0, stage_start.1, size)? {
        if backward {
            copy_backward_bytes(cpu, bus, stage_start.0, stage_start.1, size)?;
        } else {
            copy_forward_bytes(cpu, bus, stage_start.0, stage_start.1, size)?;
        }
    }
    if !backward {
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

fn try_bulk_copy_same_page(
    cpu: &mut Armv8Cpu,
    bus: &mut SystemBus,
    dst: u64,
    src: u64,
    size: u64,
) -> Result<bool, &'static str> {
    if size == 0 || !same_page_range(src, size) || !same_page_range(dst, size) {
        return Ok(false);
    }

    let src_pa = translate_or_data_fault(cpu, &mut bus.mem, src, false, "MOPS CPY bus fault")?;
    if bus.overlaps_device_range(src_pa, size as usize) {
        return Ok(false);
    }
    let mut bytes = [0; MOPS_PAGE_BYTES];
    if bus
        .mem
        .read_bytes(src_pa, &mut bytes[..size as usize])
        .is_none()
    {
        return Ok(false);
    }

    let dst_pa = translate_or_data_fault(cpu, &mut bus.mem, dst, true, "MOPS CPY bus fault")?;
    if bus.write_bytes(dst_pa, &bytes[..size as usize]).is_none() {
        return Ok(false);
    }
    cpu.clear_exclusive_range_if_overlaps(dst_pa, size);
    Ok(true)
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

fn same_page_range(va: u64, size: u64) -> bool {
    size <= PAGE_SIZE - (va & PAGE_OFFSET_MASK)
}
