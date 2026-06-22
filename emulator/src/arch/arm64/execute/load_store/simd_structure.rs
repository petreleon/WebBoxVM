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
    let total_bytes = lanes * structure_count * element_size;
    let mut bytes = [0u8; 64];
    let Some(bytes) = bytes.get_mut(..total_bytes) else {
        return Err("unsupported SIMD structure load size");
    };
    read_guest_bytes(
        cpu,
        bus,
        va,
        bytes,
        "LD4 translation fault",
        "LD4 bus fault",
    )?;

    let mut regs = [0u128; 4];
    for lane in 0..lanes {
        for reg_index in 0..structure_count {
            let mut element = 0u128;
            for byte_index in 0..element_size {
                let byte_offset = (lane * structure_count + reg_index) * element_size + byte_index;
                let byte = bytes[byte_offset] as u128;
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
    let total_bytes = lanes * structure_count * element_size;
    if !access_crosses_page(va, total_bytes as u8) {
        let mut bytes = [0u8; 64];
        let Some(bytes) = bytes.get_mut(..total_bytes) else {
            return Err("unsupported SIMD structure store size");
        };
        fill_structure_store_bytes(cpu, instr, structure_count, lanes, element_size, bytes);
        write_guest_bytes(
            cpu,
            bus,
            va,
            bytes,
            "SIMD structure store translation fault",
        )?;
        return Ok(());
    }

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

fn fill_structure_store_bytes(
    cpu: &Armv8Cpu,
    instr: Instr,
    structure_count: usize,
    lanes: usize,
    element_size: usize,
    bytes: &mut [u8],
) {
    for lane in 0..lanes {
        for reg_index in 0..structure_count {
            let value = cpu.simd[((instr.rd as usize) + reg_index) & 31];
            for byte_index in 0..element_size {
                let offset = (lane * structure_count + reg_index) * element_size + byte_index;
                let shift = lane * element_size * 8 + byte_index * 8;
                bytes[offset] = ((value >> shift) & 0xff) as u8;
            }
        }
    }
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
