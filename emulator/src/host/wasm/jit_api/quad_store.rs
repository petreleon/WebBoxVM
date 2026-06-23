use crate::constants::{PAGE_OFFSET_MASK, PAGE_SIZE};
use crate::host::wasm::{Emulator, JitPendingStore};
use crate::platform::virt::SystemBus;
use crate::runtime::Machine;
use wasm_bindgen::prelude::*;

use super::pair_store::stage_jit_pair_store_from_machine;
use super::store::translate_store;

#[wasm_bindgen]
impl Emulator {
    /// Stage four adjacent 64-bit guest RAM stores for SIMD pair transfers.
    pub fn jit_store_quad_guest(
        &mut self,
        core_id: Option<usize>,
        va: u64,
        size: u8,
        value1: u64,
        value2: u64,
        value3: u64,
        value4: u64,
    ) {
        if self.jit_helper_failed {
            return;
        }
        let core_id = core_id.unwrap_or(0);
        let stores = &mut self.jit_pending_stores;
        let result = if let Some(ref mut boot) = self.boot {
            stage_jit_quad_store_from_machine(
                &mut boot.machine,
                core_id,
                va,
                size,
                [value1, value2, value3, value4],
                stores,
            )
        } else {
            stage_jit_quad_store_from_machine(
                &mut self.machine,
                core_id,
                va,
                size,
                [value1, value2, value3, value4],
                stores,
            )
        };

        if let Err(err) = result {
            self.fail_jit_helper(err);
        }
    }
}

pub(super) fn stage_jit_quad_store_from_machine(
    machine: &mut Machine,
    core_id: usize,
    va: u64,
    size: u8,
    values: [u64; 4],
    stores: &mut Vec<JitPendingStore>,
) -> Result<(), String> {
    if size != 8 {
        return Err(format!("unsupported JIT quad store size {size}"));
    }
    if quad_access_crosses_page(va, size) {
        return stage_quad_fallback(machine, core_id, va, size, values, stores);
    }

    let (cpus, bus) = (&mut machine.cpus, &mut machine.bus);
    let cpu = cpus
        .get_mut(core_id)
        .ok_or_else(|| format!("core {core_id} does not exist"))?;
    let pa = translate_store(cpu, &mut bus.mem, va)?;
    let mut pas = [0; 4];
    for (index, lane_pa) in pas.iter_mut().enumerate() {
        *lane_pa = pa.wrapping_add(index as u64 * size as u64);
        validate_store_target(bus, *lane_pa, size)?;
    }

    let mut staged = Vec::with_capacity(4);
    for (value, lane_pa) in values.into_iter().zip(pas) {
        let bytes = value.to_le_bytes();
        staged.push(JitPendingStore::new(lane_pa, &bytes[..size as usize]));
    }
    stores.extend(staged);
    Ok(())
}

fn stage_quad_fallback(
    machine: &mut Machine,
    core_id: usize,
    va: u64,
    size: u8,
    values: [u64; 4],
    stores: &mut Vec<JitPendingStore>,
) -> Result<(), String> {
    let mut staged = Vec::with_capacity(4);
    stage_jit_pair_store_from_machine(
        machine,
        core_id,
        va,
        size,
        values[0],
        values[1],
        &mut staged,
    )?;
    stage_jit_pair_store_from_machine(
        machine,
        core_id,
        va.wrapping_add(size as u64 * 2),
        size,
        values[2],
        values[3],
        &mut staged,
    )?;
    stores.extend(staged);
    Ok(())
}

fn validate_store_target(bus: &SystemBus, pa: u64, size: u8) -> Result<(), String> {
    if bus.overlaps_device_range(pa, size as usize) || !bus.mem.contains_range(pa, size as usize) {
        return Err(format!("JIT quad store helper rejected PA 0x{pa:016x}"));
    }
    Ok(())
}

fn quad_access_crosses_page(va: u64, size: u8) -> bool {
    (va & PAGE_OFFSET_MASK) + (size as u64 * 4) > PAGE_SIZE
}
