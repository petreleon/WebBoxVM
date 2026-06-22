use super::*;

pub(in crate::arch::arm64::execute) fn read_simd_guest(
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

pub(in crate::arch::arm64::execute) fn write_simd_guest(
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
            write_guest_bytes(cpu, bus, va, &value.to_le_bytes(), err)?;
        }
        _ => return Err("unsupported SIMD store size"),
    }
    Ok(())
}

pub(in crate::arch::arm64::execute) fn read_guest(
    cpu: &mut Armv8Cpu,
    bus: &mut SystemBus,
    va: u64,
    size: u8,
    err: &'static str,
) -> Result<u64, &'static str> {
    read_guest_with_pa(cpu, bus, va, size, err).map(|(_, value)| value)
}

pub(in crate::arch::arm64::execute) fn read_guest_with_pa(
    cpu: &mut Armv8Cpu,
    bus: &mut SystemBus,
    va: u64,
    size: u8,
    err: &'static str,
) -> Result<(u64, u64), &'static str> {
    if !access_crosses_page(va, size) {
        let pa = translate_or_data_fault(cpu, &mut bus.mem, va, false, err)?;
        return read_guest_translated(cpu, bus, va, pa, size, err).map(|value| (pa, value));
    }

    let first_pa = translate_or_data_fault(cpu, &mut bus.mem, va, false, err)?;
    read_guest_translated(cpu, bus, va, first_pa, size, err).map(|value| (first_pa, value))
}

pub(in crate::arch::arm64::execute) fn read_guest_translated(
    cpu: &mut Armv8Cpu,
    bus: &mut SystemBus,
    va: u64,
    first_pa: u64,
    size: u8,
    err: &'static str,
) -> Result<u64, &'static str> {
    if !access_crosses_page(va, size) {
        return bus.read(first_pa, size).ok_or(err);
    }

    let first_byte = bus.read(first_pa, 1).ok_or(err)?;
    let mut value = 0u64;
    value |= first_byte;
    for offset in 1..size {
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

pub(in crate::arch::arm64::execute) fn write_guest(
    cpu: &mut Armv8Cpu,
    bus: &mut SystemBus,
    va: u64,
    size: u8,
    value: u64,
    err: &'static str,
) -> Result<(), &'static str> {
    if !access_crosses_page(va, size) {
        let pa = translate_or_data_fault(cpu, &mut bus.mem, va, true, err)?;
        return write_guest_translated(cpu, bus, va, pa, size, value, err);
    }

    let bytes = value.to_le_bytes();
    write_guest_bytes(cpu, bus, va, &bytes[..size as usize], err).map(|_| ())
}

pub(in crate::arch::arm64::execute) fn write_guest_translated(
    cpu: &mut Armv8Cpu,
    bus: &mut SystemBus,
    va: u64,
    first_pa: u64,
    size: u8,
    value: u64,
    err: &'static str,
) -> Result<(), &'static str> {
    if !access_crosses_page(va, size) {
        bus.write(first_pa, size, value);
        cpu.clear_exclusive_if_overlaps(first_pa, size);
        return Ok(());
    }

    let bytes = value.to_le_bytes();
    write_guest_bytes_from_first_pa(cpu, bus, va, first_pa, &bytes[..size as usize], err)
        .map(|_| ())
}

fn write_guest_bytes(
    cpu: &mut Armv8Cpu,
    bus: &mut SystemBus,
    va: u64,
    bytes: &[u8],
    err: &'static str,
) -> Result<u64, &'static str> {
    if bytes.is_empty() {
        return Ok(0);
    }

    let first_pa = translate_or_data_fault(cpu, &mut bus.mem, va, true, err)?;
    write_guest_bytes_from_first_pa(cpu, bus, va, first_pa, bytes, err)
}

fn write_guest_bytes_from_first_pa(
    cpu: &mut Armv8Cpu,
    bus: &mut SystemBus,
    va: u64,
    first_pa: u64,
    bytes: &[u8],
    err: &'static str,
) -> Result<u64, &'static str> {
    if !access_crosses_page(va, bytes.len() as u8) && bus.write_bytes(first_pa, bytes).is_some() {
        cpu.clear_exclusive_if_overlaps(first_pa, bytes.len() as u8);
        return Ok(first_pa);
    }

    let mut pas = [0u64; 16];
    pas[0] = first_pa;
    for offset in 1..bytes.len() {
        let byte_va = va.wrapping_add(offset as u64);
        pas[offset] = translate_or_data_fault(cpu, &mut bus.mem, byte_va, true, err)?;
    }

    for (offset, byte) in bytes.iter().enumerate() {
        bus.write(pas[offset], 1, *byte as u64);
        cpu.clear_exclusive_if_overlaps(pas[offset], 1);
    }
    Ok(first_pa)
}
