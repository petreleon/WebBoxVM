use super::memory_address::ADDR_LOCAL;
use super::opcodes::*;
use super::*;
use crate::arm64::{Instr, Opcode};

const JIT_LOAD_GUEST_FUNC_INDEX: u32 = 0;
const JIT_STORE_GUEST_FUNC_INDEX: u32 = 1;

impl WasmExpr {
    pub(super) fn emit_simd_memory_load(&mut self, instr: Instr) -> bool {
        match instr.op {
            Opcode::SimdLd1 => self.emit_simd_ld1(instr),
            Opcode::SimdLd1Multi => self.emit_simd_ld1_multi(instr),
            Opcode::SimdLdr => self.emit_simd_ldr(instr),
            _ => false,
        }
    }

    fn emit_simd_ld1(&mut self, instr: Instr) -> bool {
        if instr.size != 16 || instr.cond != 1 || instr.rm != 0xff || instr.imm != 0 {
            return false;
        }
        self.emit_read_base(instr.rn, true);
        self.local_set(ADDR_LOCAL);
        self.emit_load_simd_half(instr.rd, false, 0);
        self.emit_load_simd_half(instr.rd, true, 8);
        true
    }

    fn emit_simd_ld1_multi(&mut self, instr: Instr) -> bool {
        if instr.size != 16 || instr.cond != 2 || instr.rm != 0xff {
            return false;
        }
        if self.emit_memory_address(instr) != Some(false) {
            return false;
        }
        for register_index in 0..2 {
            let reg = instr.rd.wrapping_add(register_index) & 31;
            let offset = register_index as u64 * 16;
            self.emit_load_simd_half(reg, false, offset);
            self.emit_load_simd_half(reg, true, offset + 8);
        }
        true
    }

    fn emit_simd_ldr(&mut self, instr: Instr) -> bool {
        if instr.size != 16 || self.emit_memory_address(instr) != Some(false) {
            return false;
        }
        self.emit_load_simd_half(instr.rd, false, 0);
        self.emit_load_simd_half(instr.rd, true, 8);
        true
    }

    pub(super) fn emit_simd_stp(&mut self, instr: Instr) -> bool {
        if instr.size != 16 || instr.cond != 2 {
            return false;
        }
        self.emit_read_base(instr.rn, true);
        self.i64_const(instr.imm);
        self.op(OP_I64_ADD);
        self.local_set(ADDR_LOCAL);
        self.emit_store_simd_half(instr.rd, false, 0);
        self.emit_store_simd_half(instr.rd, true, 8);
        self.emit_store_simd_half(instr.rm, false, 16);
        self.emit_store_simd_half(instr.rm, true, 24);
        true
    }

    fn emit_load_simd_half(&mut self, reg: u8, high: bool, offset: u64) {
        self.emit_write_simd_half_with(reg, high, |this| {
            this.emit_guest_addr(offset);
            this.i32_const(8);
            this.call_func(JIT_LOAD_GUEST_FUNC_INDEX);
        });
    }

    fn emit_store_simd_half(&mut self, reg: u8, high: bool, offset: u64) {
        self.emit_guest_addr(offset);
        self.i32_const(8);
        self.emit_read_simd_half(reg, high);
        self.call_func(JIT_STORE_GUEST_FUNC_INDEX);
    }

    fn emit_guest_addr(&mut self, offset: u64) {
        self.local_get(ADDR_LOCAL);
        if offset != 0 {
            self.i64_const(offset);
            self.op(OP_I64_ADD);
        }
    }
}
