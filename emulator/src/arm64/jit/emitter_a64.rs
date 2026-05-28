//! ARM64→ARM64 JIT: verbatim ALU/move/logical ops at native speed.
//! Memory/branch/system ops return to dispatcher for interpreter fallback.

use crate::arm64::Armv8Cpu;
use crate::bus::SystemBus;
use crate::arm64::opcodes::Opcode;
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

        emit_prologue(&mut code);
        emit_mov(&mut code, 19, 0);
        emit_mov(&mut code, 20, 1);
        emit_mov(&mut code, 21, 2);

        for i in (0..28).step_by(2) {
            let off = i * 8;
            let ldp = 0xA9400000 | ((i as u32) << 0) | ((i as u32 + 1) << 10) | (19u32 << 5) | encode_ldp_offset(off);
            emit_a64(&mut code, ldp);
        }

        for &(_, raw) in &block.instructions {
            if can_emit_verbatim(block.instructions[compiled_count].0.op) {
                emit_a64(&mut code, raw);
                compiled_count += 1;
            } else {
                break;
            }
        }

        for i in (0..28).step_by(2) {
            let off = i * 8;
            let stp = 0xA9000000 | ((i as u32) << 0) | ((i as u32 + 1) << 10) | (19u32 << 5) | encode_ldp_offset(off);
            emit_a64(&mut code, stp);
        }
        emit_epilogue(&mut code);

        let native = NativeBlock {
            code,
            guest_instr_count: compiled_count,
            exit_pc: block.start_pc + (compiled_count as u64) * 4,
        };
        self.blocks.insert(block.start_pa, native);
        Ok(())
    }
}

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

fn emit_a64(code: &mut Vec<u8>, instr: u32) { code.extend_from_slice(&instr.to_le_bytes()); }
fn emit_mov(code: &mut Vec<u8>, rd: u8, rm: u8) {
    emit_a64(code, 0xAA0003E0 | ((rm as u32) << 16) | (rd as u32));
}
fn encode_ldp_offset(off: usize) -> u32 { ((off as u32 / 8) & 0x7F) << 15 }

fn emit_prologue(code: &mut Vec<u8>) {
    emit_a64(code, 0xA9BF7BFD); // STP X29, X30, [SP, #-16]!
    emit_a64(code, 0x910003FD); // MOV X29, SP
    emit_a64(code, 0xA9BF4FF3); // STP X19, X20
    emit_a64(code, 0xA9BF57F5); // STP X21, X22
    emit_a64(code, 0xA9BF5FF7); // STP X23, X24
    emit_a64(code, 0xA9BF67F9); // STP X25, X26
    emit_a64(code, 0xA9BF6FFB); // STP X27, X28
}
fn emit_epilogue(code: &mut Vec<u8>) {
    emit_a64(code, 0xA8C16FFB); // LDP X27, X28
    emit_a64(code, 0xA8C167F9); // LDP X25, X26
    emit_a64(code, 0xA8C15FF7); // LDP X23, X24
    emit_a64(code, 0xA8C157F5); // LDP X21, X22
    emit_a64(code, 0xA8C14FF3); // LDP X19, X20
    emit_a64(code, 0xA8C17BFD); // LDP X29, X30
    emit_a64(code, 0xD65F03C0); // RET
}
