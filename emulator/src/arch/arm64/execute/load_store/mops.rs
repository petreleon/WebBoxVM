use super::*;

mod copy;

const MOPS_PAGE_BYTES: usize = PAGE_SIZE as usize;

pub(in crate::arch::arm64::execute) fn exec_mops(
    cpu: &mut Armv8Cpu,
    bus: &mut SystemBus,
    instr: Instr,
) -> Result<(), &'static str> {
    if instr.op == Opcode::MopsSetP || instr.op == Opcode::MopsSetM || instr.op == Opcode::MopsSetE
    {
        return exec_set(cpu, bus, instr);
    }
    copy::exec_copy(cpu, bus, instr)
}

fn exec_set(cpu: &mut Armv8Cpu, bus: &mut SystemBus, instr: Instr) -> Result<(), &'static str> {
    let dst = read_reg(cpu, instr.rd, true);
    let size = set_size(cpu, instr);
    let byte = read_reg(cpu, instr.rm, true) & 0xFF;

    if !try_bulk_set_same_page(cpu, bus, dst, size, byte)? {
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
    }

    write_reg(cpu, instr.rn, 0, true);
    write_reg(cpu, instr.rd, dst.wrapping_add(size), true);
    if instr.op == Opcode::MopsSetP {
        cpu.pstate.set_nzcv(false, false, true, false);
    }
    Ok(())
}

fn try_bulk_set_same_page(
    cpu: &mut Armv8Cpu,
    bus: &mut SystemBus,
    dst: u64,
    size: u64,
    byte: u64,
) -> Result<bool, &'static str> {
    if size == 0 || size > PAGE_SIZE - (dst & PAGE_OFFSET_MASK) {
        return Ok(false);
    }

    let pa = translate_or_data_fault(cpu, &mut bus.mem, dst, true, "MOPS SET bus fault")?;
    let bytes = [byte as u8; MOPS_PAGE_BYTES];
    if bus.write_bytes(pa, &bytes[..size as usize]).is_none() {
        return Ok(false);
    }

    cpu.clear_exclusive_range_if_overlaps(pa, size);
    Ok(true)
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
