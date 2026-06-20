use super::*;

pub(in crate::arch::arm64::execute) fn simd_lane_mask(element_size: usize, shift: usize) -> u128 {
    let bits = element_size * 8;
    let element_mask = if bits == 128 {
        u128::MAX
    } else {
        (1u128 << bits) - 1
    };
    element_mask << shift
}

pub(in crate::arch::arm64::execute) fn exec_ld1_multi(
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

pub(in crate::arch::arm64::execute) fn exec_st1_multi(
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

pub(in crate::arch::arm64::execute) fn exec_ld_structure(
    cpu: &mut Armv8Cpu,
    bus: &mut SystemBus,
    va: u64,
    instr: Instr,
    structure_count: usize,
) -> Result<(), &'static str> {
    let lanes = simd_structure_lanes(instr)?;
    let element_size = 1usize << instr.cond;
    let mut regs = [0u128; 4];
    for lane in 0..lanes {
        for reg_index in 0..structure_count {
            let mut element = 0u128;
            for byte_index in 0..element_size {
                let byte_offset =
                    ((lane * structure_count + reg_index) * element_size + byte_index) as u64;
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
    for (offset, value) in regs.into_iter().take(structure_count).enumerate() {
        cpu.simd[((instr.rd as usize) + offset) & 31] = value;
    }
    Ok(())
}

pub(in crate::arch::arm64::execute) fn exec_st_structure(
    cpu: &mut Armv8Cpu,
    bus: &mut SystemBus,
    va: u64,
    instr: Instr,
    structure_count: usize,
) -> Result<(), &'static str> {
    let lanes = simd_structure_lanes(instr)?;
    let element_size = 1usize << instr.cond;
    for lane in 0..lanes {
        for reg_index in 0..structure_count {
            let value = cpu.simd[((instr.rd as usize) + reg_index) & 31];
            for byte_index in 0..element_size {
                let byte_offset =
                    ((lane * structure_count + reg_index) * element_size + byte_index) as u64;
                let pa = translate_or_data_fault(
                    cpu,
                    &mut bus.mem,
                    va.wrapping_add(byte_offset),
                    true,
                    "SIMD structure store translation fault",
                )?;
                let byte = (value >> (lane * element_size * 8 + byte_index * 8)) & 0xff;
                bus.write(pa, 1, byte as u64);
                cpu.clear_exclusive_if_overlaps(pa, 1);
            }
        }
    }
    Ok(())
}

pub(in crate::arch::arm64::execute) fn simd_structure_lanes(
    instr: Instr,
) -> Result<usize, &'static str> {
    let element_size = 1usize << instr.cond;
    if !matches!(element_size, 1 | 2 | 4 | 8) {
        return Err("unsupported SIMD structure element size");
    }
    Ok((instr.size as usize) / element_size)
}
