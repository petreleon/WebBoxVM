use super::*;

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

pub(in crate::arch::arm64::execute) fn read_guest_bytes(
    cpu: &mut Armv8Cpu,
    bus: &mut SystemBus,
    va: u64,
    bytes: &mut [u8],
    translate_err: &'static str,
    bus_err: &'static str,
) -> Result<u64, &'static str> {
    if bytes.is_empty() {
        return Ok(0);
    }

    let first_pa = translate_or_data_fault(cpu, &mut bus.mem, va, false, translate_err)?;
    read_guest_bytes_from_first_pa(cpu, bus, va, first_pa, bytes, translate_err, bus_err)
}

pub(in crate::arch::arm64::execute) fn read_guest_bytes_from_first_pa(
    cpu: &mut Armv8Cpu,
    bus: &mut SystemBus,
    va: u64,
    first_pa: u64,
    bytes: &mut [u8],
    translate_err: &'static str,
    bus_err: &'static str,
) -> Result<u64, &'static str> {
    let same_page = !access_crosses_page(va, bytes.len() as u8);
    if same_page
        && !bus.overlaps_device_range(first_pa, bytes.len())
        && bus.mem.read_bytes(first_pa, bytes).is_some()
    {
        return Ok(first_pa);
    }

    bytes[0] = bus.read(first_pa, 1).ok_or(bus_err)? as u8;
    for (offset, byte) in bytes.iter_mut().enumerate().skip(1) {
        let byte_va = va.wrapping_add(offset as u64);
        let pa = translate_or_data_fault(cpu, &mut bus.mem, byte_va, false, translate_err)?;
        *byte = bus.read(pa, 1).ok_or(bus_err)? as u8;
    }
    Ok(first_pa)
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

pub(in crate::arch::arm64::execute) fn write_guest_bytes(
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

pub(in crate::arch::arm64::execute) fn write_guest_bytes_from_first_pa(
    cpu: &mut Armv8Cpu,
    bus: &mut SystemBus,
    va: u64,
    first_pa: u64,
    bytes: &[u8],
    err: &'static str,
) -> Result<u64, &'static str> {
    if !access_crosses_page(va, bytes.len() as u8) && bus.write_bytes(first_pa, bytes).is_some() {
        cpu.clear_exclusive_range_if_overlaps(first_pa, bytes.len() as u64);
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
