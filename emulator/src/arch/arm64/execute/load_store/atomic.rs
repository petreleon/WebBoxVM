use super::*;

pub(in crate::arch::arm64::execute) fn exec_atomic(
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
        Opcode::AtomicPair => {
            let size = instr.size;
            let old_lo = bus.read(pa, size).ok_or("atomic pair bus fault")?;
            let old_hi = bus
                .read(pa + size as u64, size)
                .ok_or("atomic pair bus fault")?;
            let source_lo = read_reg(cpu, instr.rd, true);
            let source_hi = read_reg(cpu, instr.rm, true);
            let new_lo = atomic_result(instr.imm as u8, old_lo, source_lo, size)?;
            let new_hi = atomic_result(instr.imm as u8, old_hi, source_hi, size)?;
            trace_text_store(cpu, bus, &instr, "ATOMICP.0", base, pa, size, new_lo);
            bus.write(pa, size, new_lo);
            cpu.clear_exclusive_if_overlaps(pa, size);
            trace_text_store(
                cpu,
                bus,
                &instr,
                "ATOMICP.1",
                base + size as u64,
                pa + size as u64,
                size,
                new_hi,
            );
            bus.write(pa + size as u64, size, new_hi);
            cpu.clear_exclusive_if_overlaps(pa + size as u64, size);
            write_reg(cpu, instr.rd, old_lo, true);
            write_reg(cpu, instr.rm, old_hi, true);
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

pub(in crate::arch::arm64::execute) fn atomic_result(
    op: u8,
    old: u64,
    source: u64,
    size: u8,
) -> Result<u64, &'static str> {
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
