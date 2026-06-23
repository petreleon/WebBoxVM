use super::memory_address::{ADDR_LOCAL, WRITEBACK_LOCAL};
use super::opcodes::*;
use super::*;
use crate::arch::arm64::Instr;

const JIT_STORE_PAIR_GUEST_FUNC_INDEX: u32 = 6;

impl WasmExpr {
    pub(super) fn emit_simd_str(&mut self, instr: Instr) -> bool {
        if instr.size != 16 {
            return false;
        }
        let Some(writeback) = self.emit_memory_address(instr) else {
            return false;
        };
        self.emit_store_simd_q(instr.rd, 0);
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
        self.emit_store_simd_q(instr.rd, 0);
        self.emit_store_simd_q(instr.rm, 16);
        true
    }

    fn emit_store_simd_q(&mut self, reg: u8, offset: u64) {
        self.local_get(ADDR_LOCAL);
        if offset != 0 {
            self.i64_const(offset);
            self.op(OP_I64_ADD);
        }
        self.i32_const(8);
        self.emit_read_simd_half(reg, false);
        self.emit_read_simd_half(reg, true);
        self.call_func(JIT_STORE_PAIR_GUEST_FUNC_INDEX);
    }
}
