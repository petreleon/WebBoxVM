//! WebAssembly entry point: multi-core ARM64 emulator + kernel boot.
//! Compile: cargo +nightly build --target wasm64-unknown-unknown -Z build-std --features wasm
//! Bind: wasm-bindgen target/wasm64-unknown-unknown/debug/emulator.wasm --target nodejs

mod boot_api;
mod debug_api;
#[cfg(test)]
mod guard_tests;
mod io_api;
mod jit_api;
mod network_api;
#[cfg(any(test, target_arch = "wasm64"))]
mod parallel_api;
mod run_api;
mod storage_api;
#[cfg(test)]
mod tests;

use crate::arch::arm64::jit::WasmJitCpuState;
use crate::boot::BootContext;
#[cfg(target_arch = "wasm64")]
use crate::runtime::WasmDropAccess;
#[cfg(any(test, target_arch = "wasm64"))]
use crate::runtime::WasmParallelStart;
use crate::runtime::{Machine, WasmAccessControl, WasmIdleAccess};
use std::mem::ManuallyDrop;
use std::sync::Arc;
use wasm_bindgen::prelude::*;

/// Multi-core ARM64 Emulator exposed to JavaScript.
#[wasm_bindgen]
pub struct Emulator {
    // Parallel workers retain raw pointers into the active machine. Keep both
    // possible owners heap allocated so moving or freeing `Emulator` cannot
    // relocate an allocation that may need to be deliberately leaked.
    machine: ManuallyDrop<Box<Machine>>,
    boot: Option<Box<BootContext>>,
    staged_smp: bool,
    parallel_access: Arc<WasmAccessControl>,
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
    jit_prepared_block: bool,
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
    pub bytes: [u8; 32],
    pub len: u8,
}

impl Emulator {
    pub(in crate::host::wasm) fn clear_jit_side_effects(&mut self) {
        self.jit_prepared_block = false;
        self.jit_pending_exclusive_clear = None;
        self.jit_pending_exclusive_reservation = None;
        self.jit_pending_stores.clear();
    }

    pub(in crate::host::wasm) fn fail_jit_helper(&mut self, err: String) {
        self.jit_last_error = err;
        self.jit_helper_failed = true;
        self.clear_jit_side_effects();
    }

    pub(in crate::host::wasm) fn try_parallel_idle(&self) -> Result<WasmIdleAccess, &'static str> {
        self.parallel_access.try_idle()
    }

    pub(in crate::host::wasm) fn require_parallel_idle(&self) -> WasmIdleAccess {
        self.try_parallel_idle()
            .unwrap_or_else(|error| reject_parallel_machine_access(error))
    }

    #[cfg(any(test, target_arch = "wasm64"))]
    pub(in crate::host::wasm) fn require_parallel_start(&self) -> WasmParallelStart {
        self.parallel_access
            .try_parallel_start()
            .unwrap_or_else(|error| reject_parallel_machine_access(error))
    }
}

#[cold]
fn reject_parallel_machine_access(message: &str) -> ! {
    #[cfg(target_arch = "wasm64")]
    wasm_bindgen::throw_str(message);
    #[cfg(not(target_arch = "wasm64"))]
    panic!("{message}");
}

impl Drop for Emulator {
    fn drop(&mut self) {
        #[cfg(target_arch = "wasm64")]
        if matches!(self.parallel_access.claim_drop(), WasmDropAccess::Leak) {
            // Keep every possible worker target stable without dereferencing it.
            if let Some(boot) = self.boot.take() {
                std::mem::forget(boot);
            }
            return;
        }
        // No registered worker can retain a pointer into this allocation.
        unsafe { ManuallyDrop::drop(&mut self.machine) };
    }
}

#[wasm_bindgen]
impl Emulator {
    #[wasm_bindgen(constructor)]
    pub fn new(cores: Option<usize>) -> Emulator {
        Emulator {
            machine: ManuallyDrop::new(Box::new(Machine::new(cores.unwrap_or(1)))),
            boot: None,
            staged_smp: false,
            parallel_access: WasmAccessControl::new(),
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
            jit_prepared_block: false,
            jit_pending_exclusive_clear: None,
            jit_pending_exclusive_reservation: None,
            jit_pending_stores: Vec::new(),
        }
    }
}
