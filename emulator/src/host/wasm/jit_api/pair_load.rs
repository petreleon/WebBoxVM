use crate::constants::{PAGE_OFFSET_MASK, PAGE_SIZE};
use crate::host::wasm::{Emulator, JitPendingStore};
use crate::runtime::Machine;
use js_sys::Array;
use wasm_bindgen::prelude::*;

use super::load::{jit_load_guest_from_machine, read_guest_lanes, translate_load};

#[wasm_bindgen]
impl Emulator {
    /// Read adjacent guest RAM values for a generated JIT pair load.
    pub fn jit_load_pair_guest(&mut self, core_id: Option<usize>, va: u64, size: u8) -> Array {
        if self.jit_helper_failed {
            return pair_values_to_js(0, 0);
        }
        let core_id = core_id.unwrap_or(0);
        let pending_stores = &self.jit_pending_stores;
        let result = if let Some(ref mut boot) = self.boot {
            jit_load_pair_guest_from_machine(&mut boot.machine, core_id, va, size, pending_stores)
        } else {
            jit_load_pair_guest_from_machine(&mut self.machine, core_id, va, size, pending_stores)
        };

        match result {
            Ok((value1, value2)) => pair_values_to_js(value1, value2),
            Err(err) => {
                self.fail_jit_helper(err);
                pair_values_to_js(0, 0)
            }
        }
    }
}

pub(super) fn jit_load_pair_guest_from_machine(
    machine: &mut Machine,
    core_id: usize,
    va: u64,
    size: u8,
    pending_stores: &[JitPendingStore],
) -> Result<(u64, u64), String> {
    if !matches!(size, 4 | 8) {
        return Err(format!("unsupported JIT pair load size {size}"));
    }
    if pair_access_crosses_page(va, size) {
        return jit_load_pair_fallback(machine, core_id, va, size, pending_stores);
    }

    let (cpus, bus) = (&mut machine.cpus, &machine.bus);
    let cpu = cpus
        .get_mut(core_id)
        .ok_or_else(|| format!("core {core_id} does not exist"))?;
    let pa = translate_load(cpu, &bus.mem, va)?;
    let values: [u64; 2] = read_guest_lanes(&bus.mem, pending_stores, pa, size, |pa, len| {
        bus.overlaps_device_range(pa, len)
    })?;
    Ok((values[0], values[1]))
}

fn jit_load_pair_fallback(
    machine: &mut Machine,
    core_id: usize,
    va: u64,
    size: u8,
    pending_stores: &[JitPendingStore],
) -> Result<(u64, u64), String> {
    let value1 = jit_load_guest_from_machine(machine, core_id, va, size, pending_stores)?;
    let value2 = jit_load_guest_from_machine(
        machine,
        core_id,
        va.wrapping_add(size as u64),
        size,
        pending_stores,
    )?;
    Ok((value1, value2))
}

fn pair_access_crosses_page(va: u64, size: u8) -> bool {
    (va & PAGE_OFFSET_MASK) + (size as u64 * 2) > PAGE_SIZE
}

fn pair_values_to_js(value1: u64, value2: u64) -> Array {
    let values = Array::new();
    values.push(&JsValue::from(value1));
    values.push(&JsValue::from(value2));
    values
}
