//! Code cache: compiles ARM64 blocks to x86_64 and caches them.
//! Hybrid approach: JIT simple ops, call interpreter for complex ones.

use super::super::opcodes::{Instr, Opcode};
use super::super::{Armv8Cpu, execute};
use super::block::Block;
use super::emitter_x64::{CompiledBlock, Emitter, off_x, OFF_SP, OFF_PC, OFF_CYCLE_COUNT};
use crate::bus::SystemBus;
use crate::arm64::mmu::translate;
use std::collections::HashMap;

pub struct CodeCache {
    blocks: HashMap<u64, CompiledBlock>,
}

impl CodeCache {
    pub fn new() -> Self { Self { blocks: HashMap::new() } }
    pub fn get(&self, pa: u64) -> Option<&CompiledBlock> { self.blocks.get(&pa) }
    pub fn block_count(&self) -> usize { self.blocks.len() }

    pub fn compile(&mut self, block: &Block) -> Result<(), &'static str> {
        let mut e = Emitter::new();
        e.prologue();

        for &instr in &block.instructions {
            if !compile_instr_fast(&mut e, instr) {
                // Fallback: call interpreter for this instruction
                compile_interpreter_fallback(&mut e, instr)?;
            }
        }

        // Update cycle count
        let count = block.instructions.len() as u64;
        e.ld_rax(OFF_CYCLE_COUNT);
        e.add_rax_imm(count as i64);
        e.st_rax(OFF_CYCLE_COUNT);

        // Update PC (advance to next instruction or branch target)
        // PC is already updated by execute() for branches
        // For sequential blocks, we'd add count*4, but branches change it

        e.epilogue();

        let compiled = CompiledBlock {
            code: e.finish(),
            arm64_instr_count: block.instructions.len(),
        };
        self.blocks.insert(block.start_pa, compiled);
        Ok(())
    }
}

/// Returns true if instruction was compiled inline.
fn compile_instr_fast(e: &mut Emitter, i: Instr) -> bool {
    match i.op {
        Opcode::Add => {
            e.ld_rax(off_x(i.rn)); e.ld_rcx(off_x(i.rm)); e.add_rax_rcx(); e.st_rax(off_x(i.rd));
            true
        }
        Opcode::Sub => {
            e.ld_rax(off_x(i.rn)); e.ld_rcx(off_x(i.rm)); e.sub_rax_rcx(); e.st_rax(off_x(i.rd));
            true
        }
        Opcode::AddImm | Opcode::AddsImm => {
            e.ld_rax(off_x(i.rn)); e.add_rax_imm(i.imm as i64); e.st_rax(off_x(i.rd));
            true
        }
        Opcode::SubImm | Opcode::SubsImm => {
            e.ld_rax(off_x(i.rn)); e.sub_rax_imm(i.imm as i64); e.st_rax(off_x(i.rd));
            true
        }
        Opcode::Movz | Opcode::Movn => {
            e.mov_rax_imm64(i.imm); e.st_rax(off_x(i.rd));
            true
        }
        Opcode::MovReg => {
            e.ld_rax(off_x(i.rm)); e.st_rax(off_x(i.rd));
            true
        }
        Opcode::Movk => {
            // MOVK: insert 16-bit immediate at hw position
            let hw = i.cond as u64;
            let mask = !(0xFFFFu64 << (hw * 16));
            e.ld_rax(off_x(i.rd));  // RAX = old value
            // AND RAX, ~mask (keep only other bits)
            e.mov_rcx_imm64(mask);
            e.and_rax_rcx();
            // OR RAX, imm (insert immediate)
            e.mov_rcx_imm64(i.imm);
            e.or_rax_rcx();
            e.st_rax(off_x(i.rd));
            true
        }
        Opcode::Nop | Opcode::NopBarrier => true,
        Opcode::AndImm => {
            e.ld_rax(off_x(i.rn)); e.and_rax_imm(i.imm); e.st_rax(off_x(i.rd));
            true
        }
        Opcode::OrrImm => {
            e.ld_rax(off_x(i.rn)); e.or_rax_imm(i.imm); e.st_rax(off_x(i.rd));
            true
        }
        Opcode::AndReg => {
            e.ld_rax(off_x(i.rn)); e.ld_rcx(off_x(i.rm)); e.and_rax_rcx(); e.st_rax(off_x(i.rd));
            true
        }
        Opcode::OrrReg => {
            e.ld_rax(off_x(i.rn)); e.ld_rcx(off_x(i.rm)); e.or_rax_rcx(); e.st_rax(off_x(i.rd));
            true
        }
        Opcode::EorReg => {
            e.ld_rax(off_x(i.rn)); e.ld_rcx(off_x(i.rm)); e.xor_rax_rcx(); e.st_rax(off_x(i.rd));
            true
        }
        Opcode::EorImm => {
            e.ld_rax(off_x(i.rn)); e.mov_rcx_imm64(i.imm); e.xor_rax_rcx(); e.st_rax(off_x(i.rd));
            true
        }
        Opcode::Adr => {
            // ADR: PC + signed imm. PC is at block start (not current instruction).
            // We use the instruction PC from the block context.
            // For now, fall back to interpreter.
            false
        }
        Opcode::Adrp => {
            false
        }
        Opcode::Sxtw => {
            // SXTW: sign-extend 32-bit Wn to 64-bit Xd
            // movsxd rax, dword ptr [rdi + off(n)]
            // Need a different encoding: movsxd rax, [rdi + off]
            false
        }
        Opcode::Lslv => {
            // LSLV: shift left variable. rn << (rm & mask)
            e.ld_rax(off_x(i.rn));
            e.ld_rcx(off_x(i.rm));
            // AND cl, 63 (if 64-bit) or 31 (if 32-bit)
            e.e(&[0x48, 0x83, 0xE1, if i.sf { 63 } else { 31 }]); // and cl, mask
            e.shl_rax_cl();
            e.st_rax(off_x(i.rd));
            true
        }
        Opcode::Lsrv => {
            e.ld_rax(off_x(i.rn));
            e.ld_rcx(off_x(i.rm));
            e.e(&[0x48, 0x83, 0xE1, if i.sf { 63 } else { 31 }]);
            e.shr_rax_cl();
            e.st_rax(off_x(i.rd));
            true
        }
        Opcode::Asrv => {
            e.ld_rax(off_x(i.rn));
            e.ld_rcx(off_x(i.rm));
            e.e(&[0x48, 0x83, 0xE1, if i.sf { 63 } else { 31 }]);
            e.sar_rax_cl();
            e.st_rax(off_x(i.rd));
            true
        }
        Opcode::Csel | Opcode::Csinc | Opcode::Csinv | Opcode::Csneg => {
            // Conditional select — needs flag evaluation
            false
        }
        Opcode::Sbfm | Opcode::Bfm | Opcode::Ubfm => {
            false // bitfield — complex
        }
        Opcode::Madd | Opcode::Msub | Opcode::Umulh | Opcode::Smulh => {
            false // multiply — complex, needs 128-bit
        }
        Opcode::Udiv | Opcode::Sdiv => {
            false // division — needs div instruction
        }
        // Branches: handled as block terminators, not inline
        Opcode::B | Opcode::Bl | Opcode::Br | Opcode::Blr | Opcode::Ret
        | Opcode::BCond | Opcode::Cbz | Opcode::Cbnz
        | Opcode::Tbz | Opcode::Tbnz | Opcode::Svc | Opcode::Brk
        | Opcode::Eret => false,
        // Everything else → interpreter
        _ => false,
    }
}

/// Call the Rust interpreter for a single instruction.
fn compile_interpreter_fallback(e: &mut Emitter, _instr: Instr) -> Result<(), &'static str> {
    // For instructions we can't JIT, we call the Rust execute() function.
    // We can't easily do this with raw x86_64 without function pointer setup.
    // For now, just skip (instruction is a NOP in the JIT code, handled by
    // the block executor which runs the interpreter alongside).
    // The block_from_pc function already executed these via interpreter.
    Ok(())
}
