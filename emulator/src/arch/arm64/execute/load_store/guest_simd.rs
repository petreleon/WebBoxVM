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

pub(in crate::arch::arm64::execute) fn read_simd_guest_translated(
    cpu: &mut Armv8Cpu,
    bus: &mut SystemBus,
    va: u64,
    first_pa: u64,
    size: u8,
    err: &'static str,
) -> Result<u128, &'static str> {
    match size {
        1 => Ok(read_guest_translated(cpu, bus, va, first_pa, 1, err)? as u8 as u128),
        2 => Ok(read_guest_translated(cpu, bus, va, first_pa, 2, err)? as u16 as u128),
        4 => Ok(read_guest_translated(cpu, bus, va, first_pa, 4, err)? as u32 as u128),
        8 => Ok(read_guest_translated(cpu, bus, va, first_pa, 8, err)? as u128),
        16 => {
            let lo = read_guest_translated(cpu, bus, va, first_pa, 8, err)? as u128;
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

pub(in crate::arch::arm64::execute) fn write_simd_guest_translated(
    cpu: &mut Armv8Cpu,
    bus: &mut SystemBus,
    va: u64,
    first_pa: u64,
    size: u8,
    value: u128,
    err: &'static str,
) -> Result<(), &'static str> {
    match size {
        1 => write_guest_translated(cpu, bus, va, first_pa, 1, value as u8 as u64, err)?,
        2 => write_guest_translated(cpu, bus, va, first_pa, 2, value as u16 as u64, err)?,
        4 => write_guest_translated(cpu, bus, va, first_pa, 4, value as u32 as u64, err)?,
        8 => write_guest_translated(cpu, bus, va, first_pa, 8, value as u64, err)?,
        16 => {
            write_guest_bytes_from_first_pa(cpu, bus, va, first_pa, &value.to_le_bytes(), err)?;
        }
        _ => return Err("unsupported SIMD store size"),
    }
    Ok(())
}
