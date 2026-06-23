use crate::arch::arm64::{Armv8Cpu, translate_write};
use crate::constants::{PAGE_OFFSET_MASK, PAGE_SIZE};
use crate::host::wasm::{Emulator, JitPendingStore};
use crate::memory::PhysicalMemory;
use crate::runtime::Machine;
use wasm_bindgen::prelude::*;

impl JitPendingStore {
    pub(in crate::host::wasm) fn new(pa: u64, bytes: &[u8]) -> Self {
        let mut store = Self {
            pa,
            bytes: [0; 8],
            len: bytes.len() as u8,
        };
        store.bytes[..bytes.len()].copy_from_slice(bytes);
        store
    }

    fn bytes(&self) -> &[u8] {
        &self.bytes[..self.len as usize]
    }
}

#[wasm_bindgen]
impl Emulator {
    /// Stage a guest RAM store for a generated JIT block.
    pub fn jit_store_guest(&mut self, core_id: Option<usize>, va: u64, size: u8, value: u64) {
        if self.jit_helper_failed {
            return;
        }
        let core_id = core_id.unwrap_or(0);
        let stores = &mut self.jit_pending_stores;
        let result = if let Some(ref mut boot) = self.boot {
            stage_jit_store_from_machine(&mut boot.machine, core_id, va, size, value, stores)
        } else {
            stage_jit_store_from_machine(&mut self.machine, core_id, va, size, value, stores)
        };

        if let Err(err) = result {
            self.fail_jit_helper(err);
        }
    }
}

pub(super) fn stage_jit_store_from_machine(
    machine: &mut Machine,
    core_id: usize,
    va: u64,
    size: u8,
    value: u64,
    stores: &mut Vec<JitPendingStore>,
) -> Result<(), String> {
    if !matches!(size, 1 | 2 | 4 | 8) {
        return Err(format!("unsupported JIT store size {size}"));
    }
    let bytes = value.to_le_bytes();
    let (cpus, bus) = (&mut machine.cpus, &mut machine.bus);
    let cpu = cpus
        .get_mut(core_id)
        .ok_or_else(|| format!("core {core_id} does not exist"))?;

    if !access_crosses_page(va, size) {
        let pa = translate_store(cpu, &mut bus.mem, va)?;
        if bus.overlaps_device_range(pa, size as usize)
            || !bus.mem.contains_range(pa, size as usize)
        {
            return Err(format!("JIT store helper rejected PA 0x{pa:016x}"));
        }
        stores.push(JitPendingStore::new(pa, &bytes[..size as usize]));
        return Ok(());
    }

    for offset in 0..size {
        let byte_va = va.wrapping_add(offset as u64);
        let pa = translate_store(cpu, &mut bus.mem, byte_va)?;
        if bus.overlaps_device_range(pa, 1) || !bus.mem.contains_range(pa, 1) {
            return Err(format!("JIT store helper rejected PA 0x{pa:016x}"));
        }
        stores.push(JitPendingStore::new(pa, &bytes[offset as usize..][..1]));
    }
    Ok(())
}

pub(super) fn apply_jit_pending_stores(
    machine: &mut Machine,
    stores: &[JitPendingStore],
) -> Result<(), String> {
    for store in stores {
        machine
            .bus
            .mem
            .write_bytes(store.pa, store.bytes())
            .ok_or_else(|| format!("JIT pending store failed at PA 0x{:016x}", store.pa))?;
        machine.clear_exclusive_overlaps(store.pa, store.len);
    }
    Ok(())
}

pub(super) fn translate_store(
    cpu: &mut Armv8Cpu,
    mem: &mut PhysicalMemory,
    va: u64,
) -> Result<u64, String> {
    match translate_write(&cpu.sys, &mut cpu.tlb, mem, va, cpu.pstate.el()) {
        Ok(pa) => Ok(pa),
        Err(fault) => Err(format!("JIT store helper {fault:?}")),
    }
}

pub(super) fn access_crosses_page(va: u64, size: u8) -> bool {
    (va & PAGE_OFFSET_MASK) + size as u64 > PAGE_SIZE
}
