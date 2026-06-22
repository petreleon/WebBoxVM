use crate::arch::arm64::jit::{hash_raw_word, hash_seed};
use crate::arch::arm64::translate;
use crate::host::wasm::Emulator;
use crate::runtime::Machine;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
impl Emulator {
    /// Validate that a cached JIT block still matches current guest code.
    pub fn jit_validate_block(
        &mut self,
        core_id: Option<usize>,
        start_pc: u64,
        start_pa: u64,
        raw_hash: u64,
        steps: usize,
    ) -> bool {
        let core_id = core_id.unwrap_or(0);
        let result = if let Some(ref boot) = self.boot {
            validate_jit_block(&boot.machine, core_id, start_pc, start_pa, raw_hash, steps)
        } else {
            validate_jit_block(&self.machine, core_id, start_pc, start_pa, raw_hash, steps)
        };

        match result {
            Ok(()) => {
                self.jit_last_error.clear();
                true
            }
            Err(err) => {
                self.jit_last_error = err;
                false
            }
        }
    }
}

pub(super) fn validate_jit_block(
    machine: &Machine,
    core_id: usize,
    start_pc: u64,
    start_pa: u64,
    raw_hash: u64,
    steps: usize,
) -> Result<(), String> {
    if steps == 0 {
        return Err("cannot validate an empty JIT block".to_string());
    }
    let Some(cpu) = machine.cpus.get(core_id) else {
        return Err(format!("core {core_id} does not exist"));
    };
    if cpu.regs.pc != start_pc {
        return Err(format!(
            "cached JIT block starts at 0x{start_pc:016x}, current PC is 0x{:016x}",
            cpu.regs.pc
        ));
    }

    let mut tlb = cpu.tlb.clone();
    let current_pa = translate(&cpu.sys, &mut tlb, &machine.bus.mem, start_pc)
        .map_err(|_| "JIT block start translation fault".to_string())?;
    if current_pa != start_pa {
        return Err(format!(
            "cached JIT block PA changed: cached=0x{start_pa:016x} current=0x{current_pa:016x}"
        ));
    }

    for index in 1..steps {
        let pc = start_pc + index as u64 * 4;
        let addr = start_pa + index as u64 * 4;
        let current_pa = translate(&cpu.sys, &mut tlb, &machine.bus.mem, pc)
            .map_err(|_| format!("cached JIT block PC 0x{pc:016x} translation fault"))?;
        if current_pa != addr {
            return Err(format!(
                "cached JIT block PA changed at PC 0x{pc:016x}: cached=0x{addr:016x} current=0x{current_pa:016x}"
            ));
        }
    }

    let mut current_hash = hash_seed(start_pa);
    for index in 0..steps {
        let addr = start_pa + index as u64 * 4;
        let raw = machine
            .bus
            .mem
            .read(addr, 4)
            .ok_or_else(|| format!("cached JIT block word at 0x{addr:016x} is unreadable"))?;
        current_hash = hash_raw_word(current_hash, raw as u32);
    }

    if current_hash != raw_hash {
        return Err(format!(
            "cached JIT block raw hash changed: cached=0x{raw_hash:016x} current=0x{current_hash:016x}"
        ));
    }

    Ok(())
}
