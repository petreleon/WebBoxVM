use super::memory_address::{ADDR_LOCAL, WRITEBACK_LOCAL};
use super::opcodes::*;
use super::*;
use crate::arch::arm64::Instr;

impl WasmExpr {
    pub(super) fn emit_simd_str(&mut self, instr: Instr) -> bool {
        if instr.size != 16 {
            return false;
        }
        let Some(writeback) = self.emit_memory_address(instr) else {
            return false;
        };
        self.emit_store_simd_half(instr.rd, false, 0);
        self.emit_store_simd_half(instr.rd, true, 8);
        if writeback {
            self.emit_write_reg_sp_with(instr.rn, true, |this| {
                this.local_get(WRITEBACK_LOCAL);
            });
        }
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
}
