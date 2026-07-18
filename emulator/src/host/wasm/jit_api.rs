mod commit;
mod compile;
mod exclusive;
mod exclusive_load;
mod exclusive_pair;
mod finish;
mod load;
mod load_pending;
mod pair_load;
mod pair_store;
mod prepare;
mod prepared_commit;
mod quad_load;
mod quad_store;
mod state;
mod store;
mod sysreg;
mod timer;
mod validate;

use super::Emulator;
use crate::arch::arm64::jit::JIT_STATE_SIZE;
use wasm_bindgen::prelude::*;

#[cfg(test)]
mod tests;

#[wasm_bindgen]
impl Emulator {
    /// Pointer to the fixed-layout JIT CPU state inside the main wasm memory.
    ///
    /// Dynamically generated Wasm64 blocks import this same memory and receive
    /// the pointer as `run(state_ptr)`.
    pub fn jit_state_ptr(&mut self) -> u64 {
        let _access = self.require_parallel_idle();
        self.jit_state.as_mut() as *mut _ as u64
    }

    /// Size of the fixed-layout JIT CPU state in bytes.
    pub fn jit_state_size(&self) -> usize {
        let _access = self.require_parallel_idle();
        JIT_STATE_SIZE
    }

    /// Last ARM64-to-Wasm64 JIT compile/sync error.
    pub fn jit_last_error(&self) -> String {
        let _access = self.require_parallel_idle();
        self.jit_last_error.clone()
    }

    /// Guest instructions represented by the last successfully compiled JIT block.
    pub fn jit_last_block_steps(&self) -> usize {
        let _access = self.require_parallel_idle();
        self.jit_last_block_steps
    }

    /// Start PC of the last successfully compiled JIT block.
    pub fn jit_last_block_start_pc(&self) -> u64 {
        let _access = self.require_parallel_idle();
        self.jit_last_block_start_pc
    }

    /// Start physical address of the last successfully compiled JIT block.
    pub fn jit_last_block_start_pa(&self) -> u64 {
        let _access = self.require_parallel_idle();
        self.jit_last_block_start_pa
    }

    /// Exit PC of the last successfully compiled JIT block.
    pub fn jit_last_block_exit_pc(&self) -> u64 {
        let _access = self.require_parallel_idle();
        self.jit_last_block_exit_pc
    }

    /// Alternate exit PC for dynamic-exit JIT blocks.
    pub fn jit_last_block_alternate_exit_pc(&self) -> u64 {
        let _access = self.require_parallel_idle();
        self.jit_last_block_alternate_exit_pc
    }

    /// Whether the last compiled block may return one of two legal exits.
    pub fn jit_last_block_dynamic_exit(&self) -> bool {
        let _access = self.require_parallel_idle();
        self.jit_last_block_dynamic_exit
    }

    /// Packed metadata for the last successfully compiled JIT block.
    ///
    /// Order: steps, start_pc, start_pa, exit_pc, alternate_exit_pc,
    /// dynamic_exit, raw_hash, memory_generation, start_generation, end_generation.
    pub fn jit_last_block_metadata(&self) -> Vec<u64> {
        let _access = self.require_parallel_idle();
        vec![
            self.jit_last_block_steps as u64,
            self.jit_last_block_start_pc,
            self.jit_last_block_start_pa,
            self.jit_last_block_exit_pc,
            self.jit_last_block_alternate_exit_pc,
            u64::from(self.jit_last_block_dynamic_exit),
            self.jit_last_block_raw_hash,
            self.jit_last_block_memory_generation,
            self.jit_last_block_start_page_generation,
            self.jit_last_block_end_page_generation,
        ]
    }

    /// Guest exception level for the last successfully compiled JIT block.
    pub fn jit_last_block_el(&self) -> u8 {
        let _access = self.require_parallel_idle();
        self.jit_last_block_el
    }

    /// Raw-code fingerprint of the last successfully compiled JIT block.
    pub fn jit_last_block_raw_hash(&self) -> u64 {
        let _access = self.require_parallel_idle();
        self.jit_last_block_raw_hash
    }

    /// Memory generation for the first physical code page in the last block.
    pub fn jit_last_block_start_page_generation(&self) -> u64 {
        let _access = self.require_parallel_idle();
        self.jit_last_block_start_page_generation
    }

    /// Memory generation for the final physical code page in the last block.
    pub fn jit_last_block_end_page_generation(&self) -> u64 {
        let _access = self.require_parallel_idle();
        self.jit_last_block_end_page_generation
    }

    /// Whether the last compiled block uses JS guest-memory helpers.
    pub fn jit_last_block_uses_guest_helpers(&self) -> bool {
        let _access = self.require_parallel_idle();
        self.jit_last_block_uses_guest_helpers
    }

    /// Whether a generated JIT block helper rejected during the last run.
    pub fn jit_helper_failed(&self) -> bool {
        let _access = self.require_parallel_idle();
        self.jit_helper_failed
    }
}
