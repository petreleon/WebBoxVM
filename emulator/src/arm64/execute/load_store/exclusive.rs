use super::*;

pub(in crate::arm64::execute) fn exec_exclusive(
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
            let va = base;
            let pa =
                translate_or_data_fault(cpu, &mut bus.mem, va, false, "LDAR translation fault")?;
            let val = bus.read(pa, instr.size).ok_or("LDAR bus fault")?;
            write_reg(cpu, instr.rd, val, instr.sf);
            if instr.cond == 2 {
                write_reg_sp(cpu, instr.rn, base.wrapping_add(instr.imm), true);
            }
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
            let va = if instr.cond == 3 {
                base.wrapping_add(instr.imm)
            } else {
                base
            };
            let pa =
                translate_or_data_fault(cpu, &mut bus.mem, va, true, "STLR translation fault")?;
            let val = read_reg(cpu, instr.rd, instr.sf);
            trace_text_store(cpu, bus, &instr, "STLR", va, pa, instr.size, val);
            bus.write(pa, instr.size, val);
            cpu.clear_exclusive_if_overlaps(pa, instr.size);
            if instr.cond == 3 {
                write_reg_sp(cpu, instr.rn, va, true);
            }
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
