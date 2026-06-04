use super::*;

pub(in crate::arm64::execute) fn exec_ldr_str(
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
            | Opcode::Ldraa
            | Opcode::Ldrab
            | Opcode::SimdLdr
            | Opcode::SimdLd1
            | Opcode::SimdLd1Multi
            | Opcode::SimdLd1Lane
            | Opcode::SimdLd1r
            | Opcode::SimdLd2
            | Opcode::SimdLd3
            | Opcode::SimdLd4
            | Opcode::SimdLd4Single
    );

    let pa = translate_or_data_fault(cpu, &mut bus.mem, va, !is_load, "LDR/STR translation fault")?;
    if instr.op == Opcode::SimdLdr {
        cpu.simd[instr.rd as usize] = read_simd_guest(cpu, bus, va, size, "SIMD load fault")?;
    } else if instr.op == Opcode::SimdLd1 {
        cpu.simd[instr.rd as usize] = read_simd_guest(cpu, bus, va, size, "SIMD load fault")?;
    } else if instr.op == Opcode::SimdLd1Multi {
        exec_ld1_multi(cpu, bus, va, instr)?;
    } else if instr.op == Opcode::SimdLd1Lane {
        let lane = instr.imm as usize;
        let element_size = instr.cond.max(1) as usize;
        let shift = lane * element_size * 8;
        let mask = simd_lane_mask(element_size, shift);
        let value = read_guest(cpu, bus, va, element_size as u8, "SIMD lane load fault")? as u128;
        cpu.simd[instr.rd as usize] =
            (cpu.simd[instr.rd as usize] & !mask) | ((value << shift) & mask);
    } else if instr.op == Opcode::SimdLd1r {
        let element_size = instr.cond.max(1);
        let value = read_guest(cpu, bus, va, element_size, "LD1R bus fault")? as u128;
        cpu.simd[instr.rd as usize] = super::super::alu::simd_replicate_element(
            value,
            element_size as usize,
            instr.size as usize,
        );
    } else if matches!(
        instr.op,
        Opcode::SimdLd2 | Opcode::SimdLd3 | Opcode::SimdLd4
    ) {
        let structure_count = match instr.op {
            Opcode::SimdLd2 => 2,
            Opcode::SimdLd3 => 3,
            Opcode::SimdLd4 => 4,
            _ => unreachable!(),
        };
        exec_ld_structure(cpu, bus, va, instr, structure_count)?;
    } else if instr.op == Opcode::SimdLd4Single {
        let lane = (instr.imm & 0xff) as usize;
        exec_ld_structure_lane(cpu, bus, va, instr, 4, lane)?;
    } else if instr.op == Opcode::SimdStr {
        write_simd_guest(
            cpu,
            bus,
            va,
            size,
            cpu.simd[instr.rd as usize],
            "SIMD store fault",
        )?;
    } else if instr.op == Opcode::SimdSt4Single {
        let lane = (instr.imm & 0xff) as usize;
        exec_st_structure_lane(cpu, bus, va, instr, 4, lane)?;
    } else if instr.op == Opcode::SimdSt1Multi {
        exec_st1_multi(cpu, bus, va, instr)?;
    } else if instr.op == Opcode::SimdSt1Lane {
        let lane = instr.imm as usize;
        let element_size = instr.cond.max(1) as usize;
        let shift = lane * element_size * 8;
        let value =
            ((cpu.simd[instr.rd as usize] & simd_lane_mask(element_size, shift)) >> shift) as u64;
        write_guest(
            cpu,
            bus,
            va,
            element_size as u8,
            value,
            "ST1 lane bus fault",
        )?;
    } else if matches!(
        instr.op,
        Opcode::SimdSt2 | Opcode::SimdSt3 | Opcode::SimdSt4
    ) {
        let structure_count = match instr.op {
            Opcode::SimdSt2 => 2,
            Opcode::SimdSt3 => 3,
            Opcode::SimdSt4 => 4,
            _ => unreachable!(),
        };
        exec_st_structure(cpu, bus, va, instr, structure_count)?;
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

pub(in crate::arm64::execute) fn exec_ldr_lit(
    cpu: &mut Armv8Cpu,
    bus: &mut SystemBus,
    instr: Instr,
) -> Result<(), &'static str> {
    let va = branch_target(cpu.regs.pc, instr.imm);
    if instr.size != 0 {
        cpu.simd[instr.rd as usize] =
            read_simd_guest(cpu, bus, va, instr.size, "SIMD LDR literal bus fault")?;
        return Ok(());
    }

    if instr.cond == 1 {
        let val = read_guest(cpu, bus, va, 4, "LDR literal bus fault")?;
        write_reg(cpu, instr.rd, val as u32 as i32 as i64 as u64, true);
    } else {
        let size = if instr.sf { 8 } else { 4 };
        let val = read_guest(cpu, bus, va, size, "LDR literal bus fault")?;
        write_reg(cpu, instr.rd, val, instr.sf);
    }
    Ok(())
}
