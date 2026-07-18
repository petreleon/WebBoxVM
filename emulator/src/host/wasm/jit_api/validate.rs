use crate::arch::arm64::jit::{MAX_BLOCK_INSTRUCTIONS, hash_raw_word, hash_seed};
use crate::arch::arm64::translate_read_only;
use crate::constants::{INSTRUCTION_SIZE, PAGE_SHIFT};
use crate::host::wasm::Emulator;
use crate::memory::PhysicalMemory;
use crate::runtime::Machine;
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
impl Emulator {
    pub fn jit_validate_block(
        &mut self,
        core_id: Option<usize>,
        start_pc: u64,
        start_pa: u64,
        raw_hash: u64,
        memory_generation: u64,
        start_page_generation: u64,
        end_page_generation: u64,
        steps: usize,
    ) -> bool {
        let _access = self.require_parallel_idle();
        let core_id = core_id.unwrap_or(0);
        let machine = self
            .boot
            .as_ref()
            .map_or(self.machine.as_ref(), |boot| &boot.machine);
        let result = validate_jit_block(
            machine,
            core_id,
            start_pc,
            start_pa,
            raw_hash,
            memory_generation,
            start_page_generation,
            end_page_generation,
            steps,
        );

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
    memory_generation: u64,
    start_page_generation: u64,
    end_page_generation: u64,
    steps: usize,
) -> Result<(), String> {
    if steps == 0 {
        return Err("cannot validate an empty JIT block".to_string());
    }
    if steps > MAX_BLOCK_INSTRUCTIONS {
        return Err("cached JIT block exceeds maximum validation span".to_string());
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

    let current_pa = translate_read_only(&cpu.sys, Some(&cpu.tlb), &machine.bus.mem, start_pc)
        .map_err(|_| "JIT block start translation fault".to_string())?;
    if current_pa != start_pa {
        return Err(format!(
            "cached JIT block PA changed: cached=0x{start_pa:016x} current=0x{current_pa:016x}"
        ));
    }
    let end_offset = block_end_offset(steps)?;
    let end_pc = start_pc
        .checked_add(end_offset)
        .ok_or_else(|| "cached JIT block PC range overflows".to_string())?;
    let end_pa = start_pa
        .checked_add(end_offset)
        .ok_or_else(|| "cached JIT block PA range overflows".to_string())?;
    if crosses_translation_page(start_pc, start_pa, end_pc, end_pa) {
        let current_pa = translate_read_only(&cpu.sys, Some(&cpu.tlb), &machine.bus.mem, end_pc)
            .map_err(|_| format!("cached JIT block PC 0x{end_pc:016x} translation fault"))?;
        if current_pa != end_pa {
            return Err(format!(
                "cached JIT block PA changed at PC 0x{end_pc:016x}: cached=0x{end_pa:016x} current=0x{current_pa:016x}"
            ));
        }
    }
    if machine.bus.mem.generation() == memory_generation {
        return Ok(());
    }

    let (current_start_generation, current_end_generation) =
        code_page_generations_to_end(&machine.bus.mem, start_pa, end_pa)?;
    if current_start_generation == start_page_generation
        && current_end_generation == end_page_generation
    {
        return Ok(());
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

pub(super) fn code_page_generations(
    mem: &PhysicalMemory,
    start_pa: u64,
    steps: usize,
) -> Result<(u64, u64), String> {
    if steps == 0 {
        return Err("cannot inspect an empty JIT block".to_string());
    }
    let end_offset = block_end_offset(steps)?;
    let end_pa = start_pa
        .checked_add(end_offset)
        .ok_or_else(|| "cached JIT block code range overflows".to_string())?;
    code_page_generations_to_end(mem, start_pa, end_pa)
}

fn code_page_generations_to_end(
    mem: &PhysicalMemory,
    start_pa: u64,
    end_pa: u64,
) -> Result<(u64, u64), String> {
    let start = mem
        .page_generation(start_pa)
        .ok_or_else(|| format!("cached JIT block start page 0x{start_pa:016x} is unreadable"))?;
    if (start_pa >> PAGE_SHIFT) == (end_pa >> PAGE_SHIFT) {
        return Ok((start, start));
    }
    let end = mem
        .page_generation(end_pa)
        .ok_or_else(|| format!("cached JIT block end page 0x{end_pa:016x} is unreadable"))?;
    Ok((start, end))
}

fn block_end_offset(steps: usize) -> Result<u64, String> {
    (steps as u64 - 1)
        .checked_mul(INSTRUCTION_SIZE)
        .ok_or_else(|| "cached JIT block code range overflows".to_string())
}

pub(super) fn crosses_translation_page(
    start_pc: u64,
    start_pa: u64,
    end_pc: u64,
    end_pa: u64,
) -> bool {
    (start_pc >> PAGE_SHIFT) != (end_pc >> PAGE_SHIFT)
        || (start_pa >> PAGE_SHIFT) != (end_pa >> PAGE_SHIFT)
}
