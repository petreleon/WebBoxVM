use super::memory_address::ADDR_LOCAL;
use super::*;
use crate::arch::arm64::Instr;

const JIT_STORE_EXCLUSIVE_PAIR_FUNC_INDEX: u32 = 3;

impl WasmExpr {
    pub(super) fn emit_exclusive_pair_store(&mut self, instr: Instr) -> bool {
        let size = if instr.sf { 8 } else { 4 };
        if !matches!(size, 4 | 8) {
            return false;
        }

        self.emit_read_base(instr.rn, true);
        self.local_set(ADDR_LOCAL);
        self.emit_write_reg_with(instr.imm as u8, false, |this| {
            this.local_get(ADDR_LOCAL);
            this.i32_const(size as i32);
            this.emit_read_reg(instr.rd, instr.sf);
            this.emit_read_reg(instr.rm, instr.sf);
            this.call_func(JIT_STORE_EXCLUSIVE_PAIR_FUNC_INDEX);
        });
        true
    }
}
