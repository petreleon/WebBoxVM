use super::*;
use crate::constants::{PAGE_OFFSET_MASK, PAGE_SIZE};

pub(in crate::arch::arm64::execute) fn read_sve_bytes(
    cpu: &mut Armv8Cpu,
    bus: &mut SystemBus,
    va: u64,
    len: usize,
    err: &'static str,
) -> Result<[u8; 256], &'static str> {
    let mut bytes = [0; 256];
    if len == 0 {
        return Ok(bytes);
    }

    let first_pa = translate_sve_byte(cpu, bus, va, false)?;
    if !access_crosses_page(va, len)
        && !bus.overlaps_device_range(first_pa, len)
        && bus.mem.read_bytes(first_pa, &mut bytes[..len]).is_some()
    {
        return Ok(bytes);
    }

    bytes[0] = bus.read(first_pa, 1).ok_or(err)? as u8;
    for (offset, byte) in bytes.iter_mut().take(len).enumerate().skip(1) {
        let byte_va = va.wrapping_add(offset as u64);
        let pa = translate_sve_byte(cpu, bus, byte_va, false)?;
        *byte = bus.read(pa, 1).ok_or(err)? as u8;
    }
    Ok(bytes)
}

pub(in crate::arch::arm64::execute) fn write_sve_bytes(
    cpu: &mut Armv8Cpu,
    bus: &mut SystemBus,
    va: u64,
    bytes: &[u8],
    _err: &'static str,
) -> Result<(), &'static str> {
    if bytes.is_empty() {
        return Ok(());
    }

    let first_pa = translate_sve_byte(cpu, bus, va, true)?;
    if !access_crosses_page(va, bytes.len()) && bus.write_bytes(first_pa, bytes).is_some() {
        clear_exclusive_bytes(cpu, first_pa, bytes.len());
        return Ok(());
    }

    bus.write(first_pa, 1, bytes[0] as u64);
    cpu.clear_exclusive_if_overlaps(first_pa, 1);
    for (offset, byte) in bytes.iter().enumerate().skip(1) {
        let byte_va = va.wrapping_add(offset as u64);
        let pa = translate_sve_byte(cpu, bus, byte_va, true)?;
        bus.write(pa, 1, *byte as u64);
        cpu.clear_exclusive_if_overlaps(pa, 1);
    }
    Ok(())
}

fn translate_sve_byte(
    cpu: &mut Armv8Cpu,
    bus: &mut SystemBus,
    va: u64,
    write: bool,
) -> Result<u64, &'static str> {
    let result = if write {
        translate_write(&cpu.sys, &mut cpu.tlb, &mut bus.mem, va, cpu.pstate.el())
    } else {
        translate(&cpu.sys, &mut cpu.tlb, &bus.mem, va)
    };

    match result {
        Ok(pa) => Ok(pa),
        Err(
            fault @ (Fault::TranslationFault | Fault::AccessFlagFault | Fault::PermissionFault),
        ) => {
            cpu.sys.far_el1 = va;
            Err(match fault {
                Fault::TranslationFault => "translation fault",
                Fault::AccessFlagFault => "access flag fault",
                Fault::PermissionFault => "permission fault",
            })
        }
    }
}

fn access_crosses_page(va: u64, len: usize) -> bool {
    (va & PAGE_OFFSET_MASK) + len as u64 > PAGE_SIZE
}

fn clear_exclusive_bytes(cpu: &mut Armv8Cpu, pa: u64, len: usize) {
    for offset in 0..len {
        cpu.clear_exclusive_if_overlaps(pa.wrapping_add(offset as u64), 1);
    }
}
