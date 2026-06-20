use crate::arch::arm64::{Armv8Cpu, translate};
use crate::constants::{PAGE_OFFSET_MASK, PAGE_SIZE};
use crate::host::wasm::{Emulator, JitPendingStore};
use crate::memory::PhysicalMemory;
use crate::runtime::Machine;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
impl Emulator {
    /// Read guest RAM for a generated JIT block.
    ///
    /// Device ranges are rejected so a speculative JIT helper cannot consume
    /// MMIO side effects before the block passes commit checks.
    pub fn jit_load_guest(&mut self, core_id: Option<usize>, va: u64, size: u8) -> u64 {
        if self.jit_helper_failed {
            return 0;
        }
        let core_id = core_id.unwrap_or(0);
        let pending_stores = &self.jit_pending_stores;
        let result = if let Some(ref mut boot) = self.boot {
            jit_load_guest_from_machine(&mut boot.machine, core_id, va, size, pending_stores)
        } else {
            jit_load_guest_from_machine(&mut self.machine, core_id, va, size, pending_stores)
        };

        match result {
            Ok(value) => value,
            Err(err) => {
                self.jit_last_error = err;
                self.jit_helper_failed = true;
                0
            }
        }
    }
}

pub(super) fn jit_load_guest_from_machine(
    machine: &mut Machine,
    core_id: usize,
    va: u64,
    size: u8,
    pending_stores: &[JitPendingStore],
) -> Result<u64, String> {
    if !matches!(size, 1 | 2 | 4 | 8) {
        return Err(format!("unsupported JIT load size {size}"));
    }
    let (cpus, bus) = (&mut machine.cpus, &machine.bus);
    let cpu = cpus
        .get_mut(core_id)
        .ok_or_else(|| format!("core {core_id} does not exist"))?;

    if !access_crosses_page(va, size) {
        let pa = translate_load(cpu, &bus.mem, va)?;
        return read_guest_bytes(&bus.mem, pending_stores, pa, size, |pa, len| {
            bus.overlaps_device_range(pa, len)
        });
    }

    let mut value = 0u64;
    for offset in 0..size {
        let byte_va = va.wrapping_add(offset as u64);
        let pa = translate_load(cpu, &bus.mem, byte_va)?;
        if bus.overlaps_device_range(pa, 1) {
            return Err(format!("JIT load helper rejected device PA 0x{pa:016x}"));
        }
        let byte = read_guest_byte(&bus.mem, pending_stores, pa)?;
        value |= byte << (offset * 8);
    }
    Ok(value)
}

fn read_guest_bytes<F>(
    mem: &PhysicalMemory,
    pending_stores: &[JitPendingStore],
    pa: u64,
    size: u8,
    overlaps_device: F,
) -> Result<u64, String>
where
    F: Fn(u64, usize) -> bool,
{
    if overlaps_device(pa, size as usize) {
        return Err(format!("JIT load helper rejected device PA 0x{pa:016x}"));
    }
    let mut value = 0u64;
    for offset in 0..size {
        let byte = read_guest_byte(mem, pending_stores, pa.wrapping_add(offset as u64))?;
        value |= byte << (offset * 8);
    }
    Ok(value)
}

fn read_guest_byte(
    mem: &PhysicalMemory,
    pending_stores: &[JitPendingStore],
    pa: u64,
) -> Result<u64, String> {
    pending_store_byte(pending_stores, pa)
        .or_else(|| mem.read(pa, 1))
        .ok_or_else(|| format!("JIT load helper unreadable PA 0x{pa:016x}"))
}

fn pending_store_byte(pending_stores: &[JitPendingStore], pa: u64) -> Option<u64> {
    pending_stores.iter().rev().find_map(|store| {
        let offset = pa.checked_sub(store.pa)?;
        if offset < store.len as u64 {
            Some(store.bytes[offset as usize] as u64)
        } else {
            None
        }
    })
}

fn translate_load(cpu: &mut Armv8Cpu, mem: &PhysicalMemory, va: u64) -> Result<u64, String> {
    match translate(&cpu.sys, &mut cpu.tlb, mem, va) {
        Ok(pa) => Ok(pa),
        Err(fault) => {
            cpu.sys.far_el1 = va;
            Err(format!("JIT load helper {fault:?}"))
        }
    }
}

fn access_crosses_page(va: u64, size: u8) -> bool {
    (va & PAGE_OFFSET_MASK) + size as u64 > PAGE_SIZE
}
