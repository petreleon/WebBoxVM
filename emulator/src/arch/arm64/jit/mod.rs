//! JIT engine: pre-decode cache + ARM64→ARM64 native compilation.
//! Verbatim ALU/move ops execute at native speed on Apple Silicon.

use crate::arch::arm64::Armv8Cpu;
use crate::platform::virt::SystemBus;

mod block;
mod emitter_a64;
mod engine;
mod wasm64;

pub use engine::JitEngine;
pub use wasm64::{
    JIT_STATE_SIZE, Wasm64Compiler, WasmBlockModule, WasmJitCpuState, WasmJitError, hash_raw_words,
};

pub fn compile_wasm64_block_at_pc(
    cpu: &Armv8Cpu,
    bus: &SystemBus,
) -> Result<WasmBlockModule, WasmJitError> {
    let block = block::block_from_pc(cpu, bus).map_err(WasmJitError::BlockDiscovery)?;
    Wasm64Compiler::compile(&block)
}
