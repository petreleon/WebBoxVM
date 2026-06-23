use crate::arch::arm64::{Armv8Cpu, translate};
use crate::constants::{PAGE_OFFSET_MASK, PAGE_SIZE};
use crate::host::wasm::{Emulator, JitPendingStore};
use crate::memory::PhysicalMemory;
use crate::runtime::Machine;
use wasm_bindgen::prelude::*;

use super::load_pending::{pending_store_byte, pending_stores_overlap_range};

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
                self.fail_jit_helper(err);
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

pub(super) fn read_guest_bytes<F>(
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
    if !pending_stores_overlap_range(pending_stores, pa, size as usize) {
        return mem
            .read(pa, size)
            .ok_or_else(|| format!("JIT load helper unreadable PA 0x{pa:016x}"));
    }
    let mut value = 0u64;
    for offset in 0..size {
        let byte = read_guest_byte(mem, pending_stores, pa.wrapping_add(offset as u64))?;
        value |= byte << (offset * 8);
    }
    Ok(value)
}

pub(super) fn read_guest_lanes<const LANES: usize, F>(
    mem: &PhysicalMemory,
    pending_stores: &[JitPendingStore],
    pa: u64,
    size: u8,
    overlaps_device: F,
) -> Result<[u64; LANES], String>
where
    F: Fn(u64, usize) -> bool,
{
    let lane_size = size as usize;
    let len = lane_size * LANES;
    if len > 32 {
        return Err(format!("unsupported JIT lane load span {len}"));
    }
    if overlaps_device(pa, len) {
        return Err(format!("JIT load helper rejected device PA 0x{pa:016x}"));
    }

    let mut values = [0; LANES];
    if !pending_stores_overlap_range(pending_stores, pa, len) {
        let mut bytes = [0u8; 32];
        let window = &mut bytes[..len];
        mem.read_bytes(pa, window)
            .ok_or_else(|| format!("JIT load helper unreadable PA 0x{pa:016x}"))?;
        for (index, value) in values.iter_mut().enumerate() {
            *value = lane_from_bytes(&window[index * lane_size..][..lane_size]);
        }
        return Ok(values);
    }

    for (index, value) in values.iter_mut().enumerate() {
        let lane_pa = pa.wrapping_add(index as u64 * size as u64);
        for offset in 0..size {
            let byte = read_guest_byte(mem, pending_stores, lane_pa.wrapping_add(offset as u64))?;
            *value |= byte << (offset * 8);
        }
    }
    Ok(values)
}

fn lane_from_bytes(bytes: &[u8]) -> u64 {
    let mut value = 0;
    for (offset, byte) in bytes.iter().enumerate() {
        value |= (*byte as u64) << (offset * 8);
    }
    value
}

pub(super) fn read_guest_byte(
    mem: &PhysicalMemory,
    pending_stores: &[JitPendingStore],
    pa: u64,
) -> Result<u64, String> {
    pending_store_byte(pending_stores, pa)
        .or_else(|| mem.read(pa, 1))
        .ok_or_else(|| format!("JIT load helper unreadable PA 0x{pa:016x}"))
}

pub(super) fn translate_load(
    cpu: &mut Armv8Cpu,
    mem: &PhysicalMemory,
    va: u64,
) -> Result<u64, String> {
    match translate(&cpu.sys, &mut cpu.tlb, mem, va) {
        Ok(pa) => Ok(pa),
        Err(fault) => Err(format!("JIT load helper {fault:?}")),
    }
}

pub(super) fn access_crosses_page(va: u64, size: u8) -> bool {
    (va & PAGE_OFFSET_MASK) + size as u64 > PAGE_SIZE
}
