use crate::arch::arm64::{Armv8Cpu, translate_write};
use crate::host::wasm::{Emulator, JitPendingStore};
use crate::memory::PhysicalMemory;
use crate::runtime::Machine;
use wasm_bindgen::prelude::*;

use super::exclusive_pair::jit_store_exclusive_pair_from_machine;

#[wasm_bindgen]
impl Emulator {
    /// Stage an exclusive store for a generated JIT block.
    pub fn jit_store_exclusive(
        &mut self,
        core_id: Option<usize>,
        va: u64,
        size: u8,
        value: u64,
    ) -> u64 {
        let _access = self.require_parallel_idle();
        if self.jit_helper_failed {
            return 1;
        }
        let core_id = core_id.unwrap_or(0);
        let stores = &mut self.jit_pending_stores;
        let result = if let Some(ref mut boot) = self.boot {
            jit_store_exclusive_from_machine(&mut boot.machine, core_id, va, size, value, stores)
        } else {
            jit_store_exclusive_from_machine(&mut self.machine, core_id, va, size, value, stores)
        };

        match result {
            Ok(status) => {
                self.jit_pending_exclusive_clear = Some(core_id);
                status as u64
            }
            Err(err) => {
                self.fail_jit_helper(err);
                1
            }
        }
    }

    /// Stage an exclusive pair store for a generated JIT block.
    pub fn jit_store_exclusive_pair(
        &mut self,
        core_id: Option<usize>,
        va: u64,
        size: u8,
        value1: u64,
        value2: u64,
    ) -> u64 {
        let _access = self.require_parallel_idle();
        if self.jit_helper_failed {
            return 1;
        }
        let core_id = core_id.unwrap_or(0);
        let stores = &mut self.jit_pending_stores;
        let result = if let Some(ref mut boot) = self.boot {
            jit_store_exclusive_pair_from_machine(
                &mut boot.machine,
                core_id,
                va,
                size,
                value1,
                value2,
                stores,
            )
        } else {
            jit_store_exclusive_pair_from_machine(
                &mut self.machine,
                core_id,
                va,
                size,
                value1,
                value2,
                stores,
            )
        };

        match result {
            Ok(status) => {
                self.jit_pending_exclusive_clear = Some(core_id);
                status as u64
            }
            Err(err) => {
                self.fail_jit_helper(err);
                1
            }
        }
    }
}

pub(super) fn jit_store_exclusive_from_machine(
    machine: &mut Machine,
    core_id: usize,
    va: u64,
    size: u8,
    value: u64,
    stores: &mut Vec<JitPendingStore>,
) -> Result<u8, String> {
    if !matches!(size, 1 | 2 | 4 | 8) {
        return Err(format!("unsupported JIT exclusive store size {size}"));
    }
    let (cpus, bus) = (&mut machine.cpus, &mut machine.bus);
    let cpu = cpus
        .get_mut(core_id)
        .ok_or_else(|| format!("core {core_id} does not exist"))?;
    let pa = translate_store(cpu, &mut bus.mem, va)?;
    validate_store_target(bus, pa, size)?;

    if !cpu.exclusive_matches(pa, size) {
        return Ok(1);
    }

    stores.push(JitPendingStore::new(
        pa,
        &value.to_le_bytes()[..size as usize],
    ));
    Ok(0)
}

pub(super) fn apply_jit_pending_exclusive_clear(machine: &mut Machine, core_id: Option<usize>) {
    if let Some(cpu) = core_id.and_then(|id| machine.cpus.get_mut(id)) {
        cpu.clear_exclusive();
    }
}

pub(super) fn translate_store(
    cpu: &mut Armv8Cpu,
    mem: &mut PhysicalMemory,
    va: u64,
) -> Result<u64, String> {
    match translate_write(&cpu.sys, &mut cpu.tlb, mem, va, cpu.pstate.el()) {
        Ok(pa) => Ok(pa),
        Err(fault) => Err(format!("JIT exclusive store helper {fault:?}")),
    }
}

pub(super) fn validate_store_target(
    bus: &crate::platform::virt::SystemBus,
    pa: u64,
    size: u8,
) -> Result<(), String> {
    if bus.overlaps_device_range(pa, size as usize) || !bus.mem.contains_range(pa, size as usize) {
        return Err(format!(
            "JIT exclusive store helper rejected PA 0x{pa:016x}"
        ));
    }
    Ok(())
}
