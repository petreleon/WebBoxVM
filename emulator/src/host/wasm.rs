//! WebAssembly entry point: multi-core ARM64 emulator + kernel boot.
//! Compile: cargo +nightly build --target wasm64-unknown-unknown -Z build-std --features wasm
//! Bind: wasm-bindgen target/wasm64-unknown-unknown/debug/emulator.wasm --target nodejs

mod boot_api;
mod debug_api;
mod io_api;
mod jit_api;
mod network_api;
mod run_api;
mod storage_api;

use crate::arch::arm64::jit::WasmJitCpuState;
use crate::boot::BootContext;
use crate::runtime::Machine;
use wasm_bindgen::prelude::*;

/// Multi-core ARM64 Emulator exposed to JavaScript.
#[wasm_bindgen]
pub struct Emulator {
    machine: Machine,
    boot: Option<BootContext>,
    jit_state: Box<WasmJitCpuState>,
    jit_last_error: String,
    jit_last_block_steps: usize,
    jit_last_block_start_pc: u64,
    jit_last_block_start_pa: u64,
    jit_last_block_exit_pc: u64,
    jit_last_block_alternate_exit_pc: u64,
    jit_last_block_dynamic_exit: bool,
    jit_last_block_el: u8,
    jit_last_block_raw_hash: u64,
    jit_last_block_memory_generation: u64,
    jit_last_block_start_page_generation: u64,
    jit_last_block_end_page_generation: u64,
    jit_last_block_uses_guest_helpers: bool,
    jit_helper_failed: bool,
    jit_pending_exclusive_clear: Option<usize>,
    jit_pending_exclusive_reservation: Option<JitPendingExclusiveReservation>,
    jit_pending_stores: Vec<JitPendingStore>,
}

#[derive(Debug)]
pub(in crate::host::wasm) struct JitPendingExclusiveReservation {
    pub core_id: usize,
    pub pa: u64,
    pub size: u8,
}

pub(in crate::host::wasm) struct JitPendingStore {
    pub pa: u64,
    pub bytes: [u8; 8],
    pub len: u8,
}

impl Emulator {
    pub(in crate::host::wasm) fn clear_jit_side_effects(&mut self) {
        self.jit_pending_exclusive_clear = None;
        self.jit_pending_exclusive_reservation = None;
        self.jit_pending_stores.clear();
    }

    pub(in crate::host::wasm) fn fail_jit_helper(&mut self, err: String) {
        self.jit_last_error = err;
        self.jit_helper_failed = true;
        self.clear_jit_side_effects();
    }
}

#[wasm_bindgen]
impl Emulator {
    #[wasm_bindgen(constructor)]
    pub fn new(cores: Option<usize>) -> Emulator {
        Emulator {
            machine: Machine::new(cores.unwrap_or(1)),
            boot: None,
            jit_state: Box::default(),
            jit_last_error: String::new(),
            jit_last_block_steps: 0,
            jit_last_block_start_pc: 0,
            jit_last_block_start_pa: 0,
            jit_last_block_exit_pc: 0,
            jit_last_block_alternate_exit_pc: 0,
            jit_last_block_dynamic_exit: false,
            jit_last_block_el: 0,
            jit_last_block_raw_hash: 0,
            jit_last_block_memory_generation: 0,
            jit_last_block_start_page_generation: 0,
            jit_last_block_end_page_generation: 0,
            jit_last_block_uses_guest_helpers: false,
            jit_helper_failed: false,
            jit_pending_exclusive_clear: None,
            jit_pending_exclusive_reservation: None,
            jit_pending_stores: Vec::new(),
        }
    }
}
