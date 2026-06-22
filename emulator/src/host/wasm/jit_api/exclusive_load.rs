use crate::arch::arm64::{Armv8Cpu, translate};
use crate::host::wasm::{Emulator, JitPendingExclusiveReservation, JitPendingStore};
use crate::memory::PhysicalMemory;
use crate::runtime::Machine;
use wasm_bindgen::prelude::*;

use super::load::read_guest_bytes;

#[wasm_bindgen]
impl Emulator {
    /// Read RAM and stage an exclusive reservation for a generated JIT block.
    pub fn jit_load_exclusive(&mut self, core_id: Option<usize>, va: u64, size: u8) -> u64 {
        if self.jit_helper_failed {
            return 0;
        }
        let core_id = core_id.unwrap_or(0);
        let pending_stores = &self.jit_pending_stores;
        let result = if let Some(ref mut boot) = self.boot {
            jit_load_exclusive_from_machine(&mut boot.machine, core_id, va, size, pending_stores)
        } else {
            jit_load_exclusive_from_machine(&mut self.machine, core_id, va, size, pending_stores)
        };

        match result {
            Ok((value, reservation)) => {
                self.jit_pending_exclusive_reservation = Some(reservation);
                value
            }
            Err(err) => {
                self.fail_jit_helper(err);
                0
            }
        }
    }
}

pub(super) fn jit_load_exclusive_from_machine(
    machine: &mut Machine,
    core_id: usize,
    va: u64,
    size: u8,
    pending_stores: &[JitPendingStore],
) -> Result<(u64, JitPendingExclusiveReservation), String> {
    if !matches!(size, 1 | 2 | 4 | 8) {
        return Err(format!("unsupported JIT exclusive load size {size}"));
    }
    let (cpus, bus) = (&mut machine.cpus, &machine.bus);
    let cpu = cpus
        .get_mut(core_id)
        .ok_or_else(|| format!("core {core_id} does not exist"))?;
    let pa = translate_load(cpu, &bus.mem, va)?;
    let value = read_guest_bytes(&bus.mem, pending_stores, pa, size, |pa, len| {
        bus.overlaps_device_range(pa, len)
    })?;
    let reservation = JitPendingExclusiveReservation { core_id, pa, size };
    Ok((value, reservation))
}

pub(super) fn apply_jit_pending_exclusive_reservation(
    machine: &mut Machine,
    reservation: Option<JitPendingExclusiveReservation>,
) {
    if let Some(reservation) = reservation {
        if let Some(cpu) = machine.cpus.get_mut(reservation.core_id) {
            cpu.reserve_exclusive(reservation.pa, reservation.size);
        }
    }
}

fn translate_load(cpu: &mut Armv8Cpu, mem: &PhysicalMemory, va: u64) -> Result<u64, String> {
    match translate(&cpu.sys, &mut cpu.tlb, mem, va) {
        Ok(pa) => Ok(pa),
        Err(fault) => Err(format!("JIT exclusive load helper {fault:?}")),
    }
}
