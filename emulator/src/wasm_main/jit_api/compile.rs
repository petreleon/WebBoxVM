use crate::arm64::jit::compile_wasm64_block_at_pc;
use crate::wasm_main::Emulator;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
impl Emulator {
    /// Compile the block at the selected core's current PC into a Wasm64 module.
    ///
    /// Returns an empty byte vector when the current block must fall back to the
    /// interpreter. Use `jit_last_error()` for the reason.
    pub fn jit_compile_current_block(&mut self, core_id: Option<usize>) -> Vec<u8> {
        let core_id = core_id.unwrap_or(0);
        let result = if let Some(ref boot) = self.boot {
            let Some(cpu) = boot.machine.cpus.get(core_id) else {
                self.jit_last_error = format!("core {core_id} does not exist");
                return Vec::new();
            };
            compile_wasm64_block_at_pc(cpu, &boot.machine.bus)
        } else {
            let Some(cpu) = self.machine.cpus.get(core_id) else {
                self.jit_last_error = format!("core {core_id} does not exist");
                return Vec::new();
            };
            compile_wasm64_block_at_pc(cpu, &self.machine.bus)
        };

        match result {
            Ok(module) => {
                self.jit_last_error.clear();
                self.jit_last_block_steps = module.guest_instr_count;
                self.jit_last_block_start_pc = module.start_pc;
                self.jit_last_block_start_pa = module.start_pa;
                self.jit_last_block_exit_pc = module.exit_pc;
                self.jit_last_block_alternate_exit_pc = module.alternate_exit_pc;
                self.jit_last_block_dynamic_exit = module.dynamic_exit;
                self.jit_last_block_raw_hash = module.raw_hash;
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
                self.jit_last_block_raw_hash = 0;
                Vec::new()
            }
        }
    }
}
