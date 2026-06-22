use crate::arch::arm64::jit::compile_wasm64_block_at_pc;
use crate::host::wasm::Emulator;
use wasm_bindgen::prelude::*;

use super::validate::code_page_generations;

#[wasm_bindgen]
impl Emulator {
    /// Compile the block at the selected core's current PC into a Wasm64 module.
    ///
    /// Returns an empty byte vector when the current block must fall back to the
    /// interpreter. Use `jit_last_error()` for the reason.
    pub fn jit_compile_current_block(&mut self, core_id: Option<usize>) -> Vec<u8> {
        let core_id = core_id.unwrap_or(0);
        let (result, current_el) = if let Some(ref boot) = self.boot {
            let Some(cpu) = boot.machine.cpus.get(core_id) else {
                self.jit_last_error = format!("core {core_id} does not exist");
                return Vec::new();
            };
            (
                compile_wasm64_block_at_pc(cpu, &boot.machine.bus),
                cpu.pstate.el(),
            )
        } else {
            let Some(cpu) = self.machine.cpus.get(core_id) else {
                self.jit_last_error = format!("core {core_id} does not exist");
                return Vec::new();
            };
            (
                compile_wasm64_block_at_pc(cpu, &self.machine.bus),
                cpu.pstate.el(),
            )
        };

        match result {
            Ok(module) => {
                let generations = if let Some(ref boot) = self.boot {
                    code_page_generations(
                        &boot.machine.bus.mem,
                        module.start_pa,
                        module.guest_instr_count,
                    )
                } else {
                    code_page_generations(
                        &self.machine.bus.mem,
                        module.start_pa,
                        module.guest_instr_count,
                    )
                };
                let Ok((start_generation, end_generation)) = generations else {
                    self.jit_last_error = "compiled JIT block code page generation missing".into();
                    self.jit_last_block_steps = 0;
                    self.jit_last_block_start_pc = 0;
                    self.jit_last_block_start_pa = 0;
                    self.jit_last_block_raw_hash = 0;
                    self.jit_last_block_start_page_generation = 0;
                    self.jit_last_block_end_page_generation = 0;
                    return Vec::new();
                };

                self.jit_last_error.clear();
                self.jit_last_block_steps = module.guest_instr_count;
                self.jit_last_block_start_pc = module.start_pc;
                self.jit_last_block_start_pa = module.start_pa;
                self.jit_last_block_exit_pc = module.exit_pc;
                self.jit_last_block_alternate_exit_pc = module.alternate_exit_pc;
                self.jit_last_block_dynamic_exit = module.dynamic_exit;
                self.jit_last_block_el = current_el;
                self.jit_last_block_raw_hash = module.raw_hash;
                self.jit_last_block_start_page_generation = start_generation;
                self.jit_last_block_end_page_generation = end_generation;
                self.jit_last_block_uses_guest_helpers = module.uses_guest_helpers;
                module.bytes
            }
            Err(err) => {
                self.jit_last_error = err.to_string();
                self.jit_last_block_steps = 0;
                self.jit_last_block_start_pc = 0;
                self.jit_last_block_start_pa = 0;
                self.jit_last_block_exit_pc = 0;
                self.jit_last_block_alternate_exit_pc = 0;
                self.jit_last_block_dynamic_exit = false;
                self.jit_last_block_el = 0;
                self.jit_last_block_raw_hash = 0;
                self.jit_last_block_start_page_generation = 0;
                self.jit_last_block_end_page_generation = 0;
                self.jit_last_block_uses_guest_helpers = false;
                Vec::new()
            }
        }
    }
}
