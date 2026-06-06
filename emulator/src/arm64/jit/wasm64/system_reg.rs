use super::*;
use crate::arm64::Instr;
use crate::constants::SYSREG_SP_EL0;

const JIT_READ_SYSREG_FUNC_INDEX: u32 = 2;

impl WasmExpr {
    pub(super) fn emit_mrs(&mut self, instr: Instr) -> bool {
        if instr.imm as u16 != SYSREG_SP_EL0 {
            return false;
        }
        self.emit_write_reg_with(instr.rd, true, |this| {
            this.i32_const(instr.imm as i32);
            this.call_func(JIT_READ_SYSREG_FUNC_INDEX);
        });
        true
    }
}
