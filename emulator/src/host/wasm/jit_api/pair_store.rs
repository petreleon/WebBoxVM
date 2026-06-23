use crate::constants::{PAGE_OFFSET_MASK, PAGE_SIZE};
use crate::host::wasm::{Emulator, JitPendingStore};
use crate::runtime::Machine;
use wasm_bindgen::prelude::*;

use super::store::{stage_jit_store_from_machine, translate_store};

#[wasm_bindgen]
impl Emulator {
    /// Stage a scalar guest pair store for a generated JIT block.
    pub fn jit_store_pair_guest(
        &mut self,
        core_id: Option<usize>,
        va: u64,
        size: u8,
        value1: u64,
        value2: u64,
    ) {
        if self.jit_helper_failed {
            return;
        }
        let core_id = core_id.unwrap_or(0);
        let stores = &mut self.jit_pending_stores;
        let result = if let Some(ref mut boot) = self.boot {
            stage_jit_pair_store_from_machine(
                &mut boot.machine,
                core_id,
                va,
                size,
                value1,
                value2,
                stores,
            )
        } else {
            stage_jit_pair_store_from_machine(
                &mut self.machine,
                core_id,
                va,
                size,
                value1,
                value2,
                stores,
            )
        };

        if let Err(err) = result {
            self.fail_jit_helper(err);
        }
    }
}

pub(super) fn stage_jit_pair_store_from_machine(
    machine: &mut Machine,
    core_id: usize,
    va: u64,
    size: u8,
    value1: u64,
    value2: u64,
    stores: &mut Vec<JitPendingStore>,
) -> Result<(), String> {
    if !matches!(size, 4 | 8) {
        return Err(format!("unsupported JIT pair store size {size}"));
    }
    if pair_access_crosses_page(va, size) {
        return stage_pair_fallback(machine, core_id, va, size, value1, value2, stores);
    }

    let (cpus, bus) = (&mut machine.cpus, &mut machine.bus);
    let cpu = cpus
        .get_mut(core_id)
        .ok_or_else(|| format!("core {core_id} does not exist"))?;
    let pa = translate_store(cpu, &mut bus.mem, va)?;
    let total_len = size as usize * 2;
    if bus.overlaps_device_range(pa, total_len) || !bus.mem.contains_range(pa, total_len) {
        return Err(format!("JIT store helper rejected PA 0x{pa:016x}"));
    }

    let lane_size = size as usize;
    let bytes1 = value1.to_le_bytes();
    let bytes2 = value2.to_le_bytes();
    let mut bytes = [0; 16];
    bytes[..lane_size].copy_from_slice(&bytes1[..lane_size]);
    bytes[lane_size..total_len].copy_from_slice(&bytes2[..lane_size]);
    stores.push(JitPendingStore::new(pa, &bytes[..total_len]));
    Ok(())
}

fn stage_pair_fallback(
    machine: &mut Machine,
    core_id: usize,
    va: u64,
    size: u8,
    value1: u64,
    value2: u64,
    stores: &mut Vec<JitPendingStore>,
) -> Result<(), String> {
    let mut staged = Vec::with_capacity(2);
    stage_jit_store_from_machine(machine, core_id, va, size, value1, &mut staged)?;
    stage_jit_store_from_machine(
        machine,
        core_id,
        va.wrapping_add(size as u64),
        size,
        value2,
        &mut staged,
    )?;
    stores.extend(staged);
    Ok(())
}

fn pair_access_crosses_page(va: u64, size: u8) -> bool {
    (va & PAGE_OFFSET_MASK) + (size as u64 * 2) > PAGE_SIZE
}
