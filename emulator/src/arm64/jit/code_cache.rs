//! Code cache: compiles ARM64 blocks into x86_64 and caches them.

use super::super::opcodes::{Instr, Opcode};
use super::super::Armv8Cpu;
use super::block::Block;
use super::emitter_x64::{CompiledBlock, Emitter};
use crate::bus::SystemBus;
use std::collections::HashMap;

/// Cache of compiled blocks, keyed by physical address (PA) of block start.
pub struct CodeCache {
    blocks: HashMap<u64, CompiledBlock>,
}

impl CodeCache {
    pub fn new() -> Self {
        Self { blocks: HashMap::new() }
    }

    pub fn get(&self, pa: u64) -> Option<&CompiledBlock> {
        self.blocks.get(&pa)
    }

    /// Compile a block into x86_64 code and cache it.
    pub fn compile(
        &mut self,
        block: &Block,
        _cpu: &Armv8Cpu,
        _bus: &SystemBus,
    ) -> Result<(), &'static str> {
        let mut emit = Emitter::new();

        emit.prologue();

        for instr in &block.instructions {
            compile_instr(&mut emit, *instr)?;
        }

        emit.epilogue();

        let code = emit.finish();
        let compiled = CompiledBlock {
            code,
            arm64_instr_count: block.instructions.len(),
        };

        self.blocks.insert(block.start_pa, compiled);
        Ok(())
    }
}

/// Compile a single ARM64 instruction into x86_64 code.
fn compile_instr(emit: &mut Emitter, instr: Instr) -> Result<(), &'static str> {
    match instr.op {
        // ── Move instructions ──
        Opcode::Movz => {
            emit.mov_rax_imm64(instr.imm);
            emit.store_x(instr.rd);
        }
        Opcode::MovReg => {
            emit.load_x_rcx(instr.rm);
            // mov [rdi + rd_off], rcx
            let off = crate::arm64::jit::emitter_x64::reg_x_offset(instr.rd);
            if off < 128 {
                emit.emit(&[0x48, 0x89, 0x4F, off as u8]);
            } else {
                emit.emit(&[0x48, 0x89, 0x8F]);
                emit.emit_u32(off as u32);
            }
        }

        // ── Arithmetic ──
        Opcode::Add | Opcode::Adds => {
            emit.load_x(instr.rn);
            emit.load_x_rcx(instr.rm);
            emit.add_rax_rcx();
            emit.store_x(instr.rd);
        }
        Opcode::Sub | Opcode::Subs => {
            emit.load_x(instr.rn);
            emit.load_x_rcx(instr.rm);
            emit.sub_rax_rcx();
            emit.store_x(instr.rd);
        }
        Opcode::AddImm | Opcode::AddsImm => {
            emit.load_x(instr.rn);
            emit.add_rax_imm(instr.imm as i32);
            emit.store_x(instr.rd);
        }
        Opcode::SubImm | Opcode::SubsImm => {
            emit.load_x(instr.rn);
            emit.sub_rax_imm(instr.imm as i32);
            emit.store_x(instr.rd);
        }

        // ── Compare ──
        Opcode::Cmp | Opcode::CmpImm => {
            // CMP is SUB with XZR destination — sets flags only
            // For simplicity, execute via interpreter callback
            return Err("CMP: not yet JIT-compiled (needs flag emulation)");
        }

        // ── NOP ──
        Opcode::Nop | Opcode::NopBarrier => {
            // Nothing to do
        }

        // ── Branches (handled by block termination, not inline) ──
        Opcode::B | Opcode::Bl | Opcode::Br | Opcode::Blr | Opcode::Ret
        | Opcode::BCond | Opcode::Cbz | Opcode::Cbnz
        | Opcode::Tbz | Opcode::Tbnz | Opcode::Svc | Opcode::Brk
        | Opcode::Eret => {
            // Branches terminate blocks; handled by block successors
            return Err("Branch instruction: should be block terminator");
        }

        // ── Not yet JIT-compiled: fall back to interpreter ──
        _ => {
            return Err("Instruction not yet JIT-compiled");
        }
    }

    Ok(())
}
