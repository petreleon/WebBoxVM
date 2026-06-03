//! WebAssembly entry point: multi-core ARM64 emulator + kernel boot.
//! Compile: cargo +nightly build --target wasm64-unknown-unknown -Z build-std --features wasm
//! Bind: wasm-bindgen target/wasm64-unknown-unknown/debug/emulator.wasm --target nodejs

mod boot_api;
mod io_api;
mod jit_api;
mod run_api;
mod storage_api;

use crate::arm64::Machine;
use crate::arm64::jit::WasmJitCpuState;
use crate::boot::BootContext;
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
    jit_last_block_raw_hash: u64,
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
            jit_last_block_raw_hash: 0,
        }
    }
}
