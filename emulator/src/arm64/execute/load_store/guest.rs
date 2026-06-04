use super::*;

pub(in crate::arm64::execute) fn read_simd_guest(
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

pub(in crate::arm64::execute) fn write_simd_guest(
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

pub(in crate::arm64::execute) fn read_guest(
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

pub(in crate::arm64::execute) fn write_guest(
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
