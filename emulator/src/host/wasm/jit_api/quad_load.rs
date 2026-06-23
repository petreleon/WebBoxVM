use crate::constants::{PAGE_OFFSET_MASK, PAGE_SIZE};
use crate::host::wasm::{Emulator, JitPendingStore};
use crate::runtime::Machine;
use js_sys::Array;
use wasm_bindgen::prelude::*;

use super::load::{read_guest_bytes, translate_load};
use super::pair_load::jit_load_pair_guest_from_machine;

#[wasm_bindgen]
impl Emulator {
    /// Read four adjacent 64-bit guest RAM lanes for SIMD pair transfers.
    pub fn jit_load_quad_guest(&mut self, core_id: Option<usize>, va: u64, size: u8) -> Array {
        if self.jit_helper_failed {
            return quad_values_to_js([0; 4]);
        }
        let core_id = core_id.unwrap_or(0);
        let pending_stores = &self.jit_pending_stores;
        let result = if let Some(ref mut boot) = self.boot {
            jit_load_quad_guest_from_machine(&mut boot.machine, core_id, va, size, pending_stores)
        } else {
            jit_load_quad_guest_from_machine(&mut self.machine, core_id, va, size, pending_stores)
        };

        match result {
            Ok(values) => quad_values_to_js(values),
            Err(err) => {
                self.fail_jit_helper(err);
                quad_values_to_js([0; 4])
            }
        }
    }
}

pub(super) fn jit_load_quad_guest_from_machine(
    machine: &mut Machine,
    core_id: usize,
    va: u64,
    size: u8,
    pending_stores: &[JitPendingStore],
) -> Result<[u64; 4], String> {
    if size != 8 {
        return Err(format!("unsupported JIT quad load size {size}"));
    }
    if quad_access_crosses_page(va, size) {
        return jit_load_quad_fallback(machine, core_id, va, size, pending_stores);
    }

    let (cpus, bus) = (&mut machine.cpus, &machine.bus);
    let cpu = cpus
        .get_mut(core_id)
        .ok_or_else(|| format!("core {core_id} does not exist"))?;
    let pa = translate_load(cpu, &bus.mem, va)?;
    let mut values = [0; 4];
    for (index, value) in values.iter_mut().enumerate() {
        let lane_pa = pa.wrapping_add(index as u64 * size as u64);
        *value = read_guest_bytes(&bus.mem, pending_stores, lane_pa, size, |pa, len| {
            bus.overlaps_device_range(pa, len)
        })?;
    }
    Ok(values)
}

fn jit_load_quad_fallback(
    machine: &mut Machine,
    core_id: usize,
    va: u64,
    size: u8,
    pending_stores: &[JitPendingStore],
) -> Result<[u64; 4], String> {
    let first = jit_load_pair_guest_from_machine(machine, core_id, va, size, pending_stores)?;
    let second = jit_load_pair_guest_from_machine(
        machine,
        core_id,
        va.wrapping_add(size as u64 * 2),
        size,
        pending_stores,
    )?;
    Ok([first.0, first.1, second.0, second.1])
}

fn quad_access_crosses_page(va: u64, size: u8) -> bool {
    (va & PAGE_OFFSET_MASK) + (size as u64 * 4) > PAGE_SIZE
}

fn quad_values_to_js(values: [u64; 4]) -> Array {
    let array = Array::new();
    for value in values {
        array.push(&JsValue::from(value));
    }
    array
}
