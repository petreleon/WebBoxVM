//! WebAssembly entry point: multi-core ARM64 emulator via wasm-bindgen.
//! Compile: cargo +nightly build --target wasm64-unknown-unknown -Z build-std --features wasm
//! Bind: wasm-bindgen target/wasm64-unknown-unknown/debug/emulator.wasm --target nodejs

use wasm_bindgen::prelude::*;
use crate::arm64::Machine;

/// Multi-core ARM64 Emulator exposed to JavaScript.
#[wasm_bindgen]
pub struct Emulator {
    machine: Machine,
}

#[wasm_bindgen]
impl Emulator {
    /// Create an emulator with `cores` CPU cores (default: 1).
    #[wasm_bindgen(constructor)]
    pub fn new(cores: Option<usize>) -> Emulator {
        Emulator {
            machine: Machine::new(cores.unwrap_or(1)),
        }
    }

    /// Load raw ARM64 instructions into RAM at 0x4000_0000.
    pub fn load(&mut self, instructions: Vec<u32>) {
        for (i, &word) in instructions.iter().enumerate() {
            self.machine.bus.write(0x4000_0000 + (i as u64) * 4, 4, word as u64);
        }
    }

    /// Run up to `max_steps` instructions across all cores (round-robin).
    /// Returns a status string.
    pub fn run_steps(&mut self, max_steps: usize) -> String {
        let executed = self.machine.run(max_steps);
        format!("OK: {} steps", executed)
    }

    /// Get register Xn of core `core_id` (default: 0).
    pub fn reg(&self, n: u8, core_id: Option<usize>) -> u64 {
        let cid = core_id.unwrap_or(0);
        if cid < self.machine.cpus.len() {
            self.machine.cpus[cid].regs.x(n)
        } else {
            0
        }
    }

    /// Get UART output string.
    pub fn uart_output(&self) -> String {
        self.machine.bus.uart.output_string()
    }

    /// Set program counter for core `core_id`.
    pub fn set_pc(&mut self, pc: u64, core_id: Option<usize>) {
        let cid = core_id.unwrap_or(0);
        if cid < self.machine.cpus.len() {
            self.machine.cpus[cid].regs.pc = pc;
        }
    }

    /// Get total steps executed across all cores.
    pub fn total_steps(&self) -> u64 { self.machine.total_steps }

    /// Get the number of cores.
    pub fn num_cores(&self) -> usize { self.machine.cpus.len() }

    /// Get the active core index.
    pub fn active_core(&self) -> usize { self.machine.active_core }
}
