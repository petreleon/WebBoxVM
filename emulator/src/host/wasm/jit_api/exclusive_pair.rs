use crate::constants::{PAGE_OFFSET_MASK, PAGE_SIZE};
use crate::host::wasm::JitPendingStore;
use crate::runtime::Machine;

use super::exclusive::{translate_store, validate_store_target};

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
    let pa2 = if pair_access_crosses_page(va, size) {
        translate_store(cpu, &mut bus.mem, va.wrapping_add(size as u64))?
    } else {
        pa1.wrapping_add(size as u64)
    };
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

fn pair_access_crosses_page(va: u64, size: u8) -> bool {
    (va & PAGE_OFFSET_MASK) + (size as u64 * 2) > PAGE_SIZE
}
