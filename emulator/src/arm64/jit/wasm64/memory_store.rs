use super::*;
use crate::arm64::Instr;

use super::memory_address::{ADDR_LOCAL, VALUE_LOCAL, WRITEBACK_LOCAL};

const JIT_STORE_GUEST_FUNC_INDEX: u32 = 1;

impl WasmExpr {
    pub(super) fn emit_memory_store(&mut self, instr: Instr) -> bool {
        if !matches!(instr.size, 1 | 2 | 4 | 8) {
            return false;
        }
        let Some(writeback) = self.emit_memory_address(instr) else {
            return false;
        };

        self.emit_read_reg(instr.rd, instr.sf);
        self.local_set(VALUE_LOCAL);
        self.local_get(ADDR_LOCAL);
        self.i32_const(instr.size as i32);
        self.local_get(VALUE_LOCAL);
        self.call_func(JIT_STORE_GUEST_FUNC_INDEX);
        if writeback {
            self.emit_write_reg_sp_with(instr.rn, true, |this| {
                this.local_get(WRITEBACK_LOCAL);
            });
        }
        true
    }
}
