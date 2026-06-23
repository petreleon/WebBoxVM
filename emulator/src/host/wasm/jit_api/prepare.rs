use crate::arch::arm64::jit::WasmJitCpuState;
use crate::host::wasm::Emulator;
use crate::runtime::Machine;
use wasm_bindgen::prelude::*;

use super::commit::can_commit_jit_block_now;
use super::validate::validate_jit_block;

#[wasm_bindgen]
impl Emulator {
    /// Preflight, validate, and copy one core into the JIT state buffer.
    pub fn jit_prepare_cached_block(
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
        let core_id = core_id.unwrap_or(0);
        self.jit_helper_failed = false;
        self.clear_jit_side_effects();

        let result = if let Some(ref mut boot) = self.boot {
            prepare_jit_block(
                &mut boot.machine,
                &mut self.jit_state,
                core_id,
                start_pc,
                start_pa,
                raw_hash,
                memory_generation,
                start_page_generation,
                end_page_generation,
                steps,
            )
        } else {
            prepare_jit_block(
                &mut self.machine,
                &mut self.jit_state,
                core_id,
                start_pc,
                start_pa,
                raw_hash,
                memory_generation,
                start_page_generation,
                end_page_generation,
                steps,
            )
        };

        match result {
            Ok(()) => {
                self.jit_last_error.clear();
                self.jit_prepared_block = true;
                true
            }
            Err(err) => {
                self.jit_last_error = err;
                false
            }
        }
    }
}

pub(super) fn prepare_jit_block(
    machine: &mut Machine,
    state: &mut WasmJitCpuState,
    core_id: usize,
    start_pc: u64,
    start_pa: u64,
    raw_hash: u64,
    memory_generation: u64,
    start_page_generation: u64,
    end_page_generation: u64,
    steps: usize,
) -> Result<(), String> {
    can_commit_jit_block_now(machine, core_id, steps)?;
    validate_jit_block(
        machine,
        core_id,
        start_pc,
        start_pa,
        raw_hash,
        memory_generation,
        start_page_generation,
        end_page_generation,
        steps,
    )?;

    let Some(cpu) = machine.cpus.get(core_id) else {
        return Err(format!("core {core_id} does not exist"));
    };
    state.copy_from_cpu(cpu);
    Ok(())
}
