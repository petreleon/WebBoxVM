//! ARM64-to-Wasm64 block compiler.
//!
//! The browser JIT path cannot rely on native executable memory. Instead it
//! emits small WebAssembly modules that import the existing Memory64 instance
//! and operate on a fixed CPU-state buffer:
//!
//! ```text
//! export run(state_ptr: i64) -> i64
//! ```
//!
//! This backend is intentionally conservative. It only compiles straight-line,
//! register-only instructions whose semantics are independent of memory, MMIO,
//! exceptions, timers, and system registers. Unsupported instructions are hard
//! fallback boundaries.

use super::block::Block;
use crate::arm64::Opcode;
use crate::constants::{SP_REGISTER_INDEX, ZERO_REGISTER_INDEX};

mod bitfield;
mod cmp_flags;
mod cond_select;
mod condition;
mod emit_instr;
mod encoding;
mod expr;
mod hash;
mod helpers;
mod module_builder;
mod opcodes;
mod rev;
mod state;
mod terminal_branch;
mod types;

#[cfg(test)]
mod tests;

use expr::WasmExpr;
pub use hash::hash_raw_words;
use hash::{hash_raw_word, hash_seed};
use helpers::{can_emit_shift, logical_opcode, reg_offset};
use module_builder::build_module;
pub use state::{
    JIT_STATE_PC_OFFSET, JIT_STATE_PSTATE_OFFSET, JIT_STATE_SIZE, JIT_STATE_SP_OFFSET,
    JIT_STATE_X_OFFSET, WasmJitCpuState,
};
pub use types::{WasmBlockModule, WasmJitError};

pub struct Wasm64Compiler;

impl Wasm64Compiler {
    pub fn compile(block: &Block) -> Result<WasmBlockModule, WasmJitError> {
        if block.instructions.is_empty() {
            return Err(WasmJitError::EmptyBlock);
        }

        let mut body = WasmExpr::new();
        let mut compiled = 0usize;
        let mut dynamic_exit = false;
        let mut exit_pc = block.start_pc;
        let mut alternate_exit_pc = block.start_pc;
        let mut raw_hash = hash_seed(block.start_pa);

        for (index, &(instr, raw)) in block.instructions.iter().enumerate() {
            let expected_pa = block.start_pa + index as u64 * 4;
            let pc = block.start_pc + index as u64 * 4;
            if block.instruction_pas.get(index).copied() != Some(expected_pa) {
                if compiled == 0 {
                    return Err(WasmJitError::BlockDiscovery(
                        "non-contiguous physical block",
                    ));
                }
                break;
            }
            if let Some(exits) = body.emit_terminal_branch(instr, pc) {
                dynamic_exit = true;
                exit_pc = exits.fallthrough;
                alternate_exit_pc = exits.target;
                raw_hash = hash_raw_word(raw_hash, raw);
                compiled += 1;
                break;
            }
            if !body.emit_instr(instr, pc) {
                if compiled == 0 {
                    return Err(WasmJitError::UnsupportedFirstOpcode(instr.op));
                }
                break;
            }
            raw_hash = hash_raw_word(raw_hash, raw);
            compiled += 1;
        }

        if !dynamic_exit {
            exit_pc = block.start_pc + compiled as u64 * 4;
            alternate_exit_pc = exit_pc;
            body.emit_store_const(JIT_STATE_PC_OFFSET, exit_pc);
            body.i64_const(exit_pc);
        }
        body.end();

        Ok(WasmBlockModule {
            start_pc: block.start_pc,
            start_pa: block.start_pa,
            exit_pc,
            alternate_exit_pc,
            dynamic_exit,
            guest_instr_count: compiled,
            raw_hash,
            bytes: build_module(body.into_bytes()),
        })
    }
}
