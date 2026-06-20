use super::*;

pub(in crate::arch::arm64::execute) fn exec_ld_structure_lane(
    cpu: &mut Armv8Cpu,
    bus: &mut SystemBus,
    va: u64,
    instr: Instr,
    structure_count: usize,
    lane: usize,
) -> Result<(), &'static str> {
    let element_size = instr.cond.max(1) as usize;
    let shift = lane * element_size * 8;
    let mask = simd_lane_mask(element_size, shift);
    for reg_index in 0..structure_count {
        let byte_offset = (reg_index * element_size) as u64;
        let value = read_guest(
            cpu,
            bus,
            va.wrapping_add(byte_offset),
            element_size as u8,
            "SIMD structure lane load fault",
        )? as u128;
        let reg = ((instr.rd as usize) + reg_index) & 31;
        cpu.simd[reg] = (cpu.simd[reg] & !mask) | ((value << shift) & mask);
    }
    Ok(())
}

pub(in crate::arch::arm64::execute) fn exec_st_structure_lane(
    cpu: &mut Armv8Cpu,
    bus: &mut SystemBus,
    va: u64,
    instr: Instr,
    structure_count: usize,
    lane: usize,
) -> Result<(), &'static str> {
    let element_size = instr.cond.max(1) as usize;
    for reg_index in 0..structure_count {
        let value = cpu.simd[((instr.rd as usize) + reg_index) & 31];
        for byte_index in 0..element_size {
            let byte_offset = (reg_index * element_size + byte_index) as u64;
            let pa = translate_or_data_fault(
                cpu,
                &mut bus.mem,
                va.wrapping_add(byte_offset),
                true,
                "SIMD structure lane store translation fault",
            )?;
            let shift = lane * element_size * 8 + byte_index * 8;
            bus.write(pa, 1, ((value >> shift) & 0xff) as u64);
            cpu.clear_exclusive_if_overlaps(pa, 1);
        }
    }
    Ok(())
}
