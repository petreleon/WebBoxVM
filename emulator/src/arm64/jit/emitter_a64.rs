//! ARM64→ARM64 JIT: verbatim ALU/move/logical ops at native speed.
//! Memory/branch/system ops return to dispatcher for interpreter fallback.

use crate::arm64::Armv8Cpu;
use crate::bus::SystemBus;
use crate::arm64::opcodes::{Instr, Opcode};
use super::block::Block;
use std::collections::HashMap;

pub struct NativeBlock {
    code: Vec<u8>,
    pub guest_instr_count: usize,
    pub exit_pc: u64,
}

impl NativeBlock {
    pub unsafe fn execute(&self, cpu: &mut Armv8Cpu, bus: &mut SystemBus) {
        let ram_ptr = bus.mem.ram_data();
        type JitFn = extern "C" fn(u64, u64, u64);
        let jit: JitFn = std::mem::transmute(self.code.as_ptr());
        jit(cpu as *mut _ as u64, bus as *mut _ as u64, ram_ptr as u64);
    }
}

pub struct A64Compiler {
    blocks: HashMap<u64, NativeBlock>,
}

impl A64Compiler {
    pub fn new() -> Self { Self { blocks: HashMap::new() } }

    pub fn get(&self, pa: u64) -> Option<&NativeBlock> { self.blocks.get(&pa) }
    pub fn block_count(&self) -> usize { self.blocks.len() }

    pub fn compile(&mut self, block: &Block, _cpu: &Armv8Cpu, _bus: &SystemBus) -> Result<(), &'static str> {
        let mut code: Vec<u8> = Vec::new();
        let mut compiled_count = 0usize;

        // Prologue
        emit_prologue(&mut code);

        // X19 = cpu ptr (arg0), X20 = bus ptr (arg1), X21 = ram base (arg2)
        emit_mov(&mut code, 19, 0);
        emit_mov(&mut code, 20, 1);
        emit_mov(&mut code, 21, 2);

        // Load guest registers X0-X28 from Armv8Cpu (X19 points to cpu)
        // X19 + 0 = X0, X19 + 8 = X1, ... X19 + 224 = X28
        // Use LDP to load 2 registers at once
        for i in (0..28).step_by(2) {
            let off = i * 8;
            // LDP Xi, Xi+1, [X19, #off]
            let ldp = 0xA9400000 | ((i as u32) << 0) | ((i as u32 + 1) << 10) | (19u32 << 5) | encode_ldp_offset(off);
            emit_a64(&mut code, ldp);
        }
        // Load X29, X30 separately (if needed)
        emit_ldr_imm(&mut code, 29, 19, 29 * 8);
        emit_ldr_imm(&mut code, 30, 19, 30 * 8);

        // Load SP from guest context: LDR Xtmp, [X19, #SP_OFFSET]; MOV SP, Xtmp
        // SP offset in Armv8Cpu: regs.sp is at index 31 → offset 31*8 = 248
        emit_ldr_imm(&mut code, 0, 19, 31 * 8); // X0 = guest SP (temporarily clobbers X0)
        // We already loaded X0 above, so we need to reload it after setting SP
        // Actually SP setup needs care — let's skip SP for now and use interpreter SP
        // Instead: load guest SP into Xtmp, then MOV SP, Xtmp
        // But we need a scratch register. Use X15 (temporary)
        emit_ldr_imm(&mut code, 15, 19, 31 * 8); // X15 = guest SP
        // MOV SP, X15  → 0x910003FF | (15 << 5) → wait, that's ADD SP, X15, #0
        emit_a64(&mut code, 0x910003FF | (15u32 << 5)); // MOV SP, X15

        for &(instr, raw) in &block.instructions {
            if can_emit_verbatim(instr.op) {
                emit_a64(&mut code, raw);
                compiled_count += 1;
            } else {
                break;
            }
        }

        // Epilogue: store guest registers back, restore host frame
        emit_epilogue(&mut code);

        let start_pa = block.start_pa;
        let native = NativeBlock {
            code,
            guest_instr_count: compiled_count,
            exit_pc: block.start_pc + (compiled_count as u64) * 4,
        };
        self.blocks.insert(start_pa, native);
        Ok(())
    }
}

/// Returns true if this opcode can be JIT'd verbatim on ARM64 host.
/// These are instructions where the guest encoding = host encoding.
fn can_emit_verbatim(op: Opcode) -> bool {
    matches!(
        op,
        Opcode::Add | Opcode::Sub | Opcode::Adds | Opcode::Subs
        | Opcode::AddImm | Opcode::SubImm | Opcode::AddsImm | Opcode::SubsImm
        | Opcode::AndReg | Opcode::OrrReg | Opcode::EorReg | Opcode::AndsReg
        | Opcode::AndImm | Opcode::OrrImm | Opcode::EorImm | Opcode::AndsImm
        | Opcode::MovReg | Opcode::Movz | Opcode::Movk | Opcode::Movn
        | Opcode::Cmp | Opcode::CmpImm
        | Opcode::Sxtw | Opcode::Sbfm | Opcode::Bfm | Opcode::Ubfm
        | Opcode::Csel | Opcode::Csinc | Opcode::Csinv | Opcode::Csneg | Opcode::Ccmp
        | Opcode::Udiv | Opcode::Sdiv
        | Opcode::Madd | Opcode::Msub | Opcode::Umulh | Opcode::Smulh
        | Opcode::Lslv | Opcode::Lsrv | Opcode::Asrv | Opcode::Rorv
        | Opcode::Rev | Opcode::Rev32 | Opcode::Rev16 | Opcode::Rbit | Opcode::Clz
        | Opcode::Nop | Opcode::NopBarrier
        | Opcode::AddExt | Opcode::SubExt | Opcode::AddsExt | Opcode::SubsExt
    )
}

fn emit_a64(code: &mut Vec<u8>, instr: u32) {
    code.extend_from_slice(&instr.to_le_bytes());
}

fn emit_mov(code: &mut Vec<u8>, rd: u8, rm: u8) {
    emit_a64(code, 0xAA0003E0 | ((rm as u32) << 16) | (rd as u32));
}

// LDR Xd, [Xn, #offset] where offset is unsigned and < 32768
fn emit_ldr_imm(code: &mut Vec<u8>, rd: u8, rn: u8, offset: usize) {
    // LDR Xd, [Xn, #offset]: 0xF9400000 | (offset/8)<<10 | (rn)<<5 | rd
    let imm12 = (offset as u32 / 8) & 0xFFF;
    emit_a64(code, 0xF9400000 | (imm12 << 10) | ((rn as u32) << 5) | (rd as u32));
}

// STR Xd, [Xn, #offset]
fn emit_str_imm(code: &mut Vec<u8>, rd: u8, rn: u8, offset: usize) {
    let imm12 = (offset as u32 / 8) & 0xFFF;
    emit_a64(code, 0xF9000000 | (imm12 << 10) | ((rn as u32) << 5) | (rd as u32));
}

// Encode LDP/STP immediate offset (7-bit signed, scaled by 8)
fn encode_ldp_offset(off: usize) -> u32 {
    ((off as u32 / 8) & 0x7F) << 15
}

fn emit_prologue(code: &mut Vec<u8>) {
    // STP X29, X30, [SP, #-16]!
    emit_a64(code, 0xA9BF7BFD);
    // MOV X29, SP
    emit_a64(code, 0x910003FD);
    // Save X19-X28 (callee-saved, 5 pairs)
    emit_a64(code, 0xA9BF4FF3); // STP X19, X20, [SP, #-16]!
    emit_a64(code, 0xA9BF57F5); // STP X21, X22, [SP, #-16]!
    emit_a64(code, 0xA9BF5FF7); // STP X23, X24, [SP, #-16]!
    emit_a64(code, 0xA9BF67F9); // STP X25, X26, [SP, #-16]!
    emit_a64(code, 0xA9BF6FFB); // STP X27, X28, [SP, #-16]!
}

fn emit_epilogue(code: &mut Vec<u8>) {
    // Restore X27-X19, X29, X30 and return
    emit_a64(code, 0xA8C16FFB); // LDP X27, X28, [SP], #16
    emit_a64(code, 0xA8C167F9); // LDP X25, X26, [SP], #16
    emit_a64(code, 0xA8C15FF7); // LDP X23, X24, [SP], #16
    emit_a64(code, 0xA8C157F5); // LDP X21, X22, [SP], #16
    emit_a64(code, 0xA8C14FF3); // LDP X19, X20, [SP], #16
    emit_a64(code, 0xA8C17BFD); // LDP X29, X30, [SP], #16
    emit_a64(code, 0xD65F03C0); // RET
}
