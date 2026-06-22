use crate::arch::arm64::{Armv8Cpu, translate_write};
use crate::host::wasm::{Emulator, JitPendingStore};
use crate::memory::PhysicalMemory;
use crate::runtime::Machine;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
impl Emulator {
    /// Stage an exclusive pair store for a generated JIT block.
    pub fn jit_store_exclusive_pair(
        &mut self,
        core_id: Option<usize>,
        va: u64,
        size: u8,
        value1: u64,
        value2: u64,
    ) -> u64 {
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

pub(super) fn jit_store_exclusive_pair_from_machine(
    machine: &mut Machine,
    core_id: usize,
    va: u64,
    size: u8,
    value1: u64,
    value2: u64,
    stores: &mut Vec<JitPendingStore>,
) -> Result<u8, String> {
    if !matches!(size, 4 | 8) {
        return Err(format!("unsupported JIT exclusive pair size {size}"));
    }
    let (cpus, bus) = (&mut machine.cpus, &mut machine.bus);
    let cpu = cpus
        .get_mut(core_id)
        .ok_or_else(|| format!("core {core_id} does not exist"))?;
    let pa1 = translate_store(cpu, &mut bus.mem, va)?;
    let pa2 = translate_store(cpu, &mut bus.mem, va.wrapping_add(size as u64))?;
    validate_store_target(bus, pa1, size)?;
    validate_store_target(bus, pa2, size)?;

    let total_size = size * 2;
    if !cpu.exclusive_matches(pa1, total_size) {
        return Ok(1);
    }

    stores.push(JitPendingStore::new(
        pa1,
        &value1.to_le_bytes()[..size as usize],
    ));
    stores.push(JitPendingStore::new(
        pa2,
        &value2.to_le_bytes()[..size as usize],
    ));
    Ok(0)
}

pub(super) fn apply_jit_pending_exclusive_clear(machine: &mut Machine, core_id: Option<usize>) {
    if let Some(cpu) = core_id.and_then(|id| machine.cpus.get_mut(id)) {
        cpu.clear_exclusive();
    }
}

fn translate_store(cpu: &mut Armv8Cpu, mem: &mut PhysicalMemory, va: u64) -> Result<u64, String> {
    match translate_write(&cpu.sys, &mut cpu.tlb, mem, va, cpu.pstate.el()) {
        Ok(pa) => Ok(pa),
        Err(fault) => Err(format!("JIT exclusive store helper {fault:?}")),
    }
}

fn validate_store_target(
    bus: &crate::platform::virt::SystemBus,
    pa: u64,
    size: u8,
) -> Result<(), String> {
    if bus.overlaps_device_range(pa, size as usize) || bus.mem.read(pa, size).is_none() {
        return Err(format!(
            "JIT exclusive store helper rejected PA 0x{pa:016x}"
        ));
    }
    Ok(())
}
