use super::opcodes::*;
use super::*;
use crate::arch::arm64::{Instr, Opcode};

use super::memory_address::{ADDR_LOCAL, VALUE_LOCAL, WRITEBACK_LOCAL};

const JIT_LOAD_GUEST_FUNC_INDEX: u32 = 0;

impl WasmExpr {
    pub(super) fn emit_memory_load(&mut self, instr: Instr) -> bool {
        if !matches!(instr.size, 1 | 2 | 4 | 8) {
            return false;
        }
        let Some(writeback) = self.emit_memory_address(instr) else {
            return false;
        };

        self.local_get(ADDR_LOCAL);
        self.i32_const(instr.size as i32);
        self.call_func(JIT_LOAD_GUEST_FUNC_INDEX);
        if instr.op == Opcode::LdrSign {
            self.emit_load_sign_extend(instr.size);
        }
        self.local_set(VALUE_LOCAL);
        self.emit_write_reg_with(instr.rd, instr.sf, |this| this.local_get(VALUE_LOCAL));
        if writeback {
            self.emit_write_reg_sp_with(instr.rn, true, |this| {
                this.local_get(WRITEBACK_LOCAL);
            });
        }
        true
    }

    fn emit_load_sign_extend(&mut self, size: u8) {
        let shift = match size {
            1 => 56,
            2 => 48,
            4 => 32,
            _ => 0,
        };
        if shift == 0 {
            return;
        }
        self.i64_const(shift);
        self.op(OP_I64_SHL);
        self.i64_const(shift);
        self.op(OP_I64_SHR_S);
    }
}
