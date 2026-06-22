use super::*;
use crate::arch::arm64::Instr;
use crate::constants::{
    SYSREG_CNTPCT_EL0, SYSREG_CNTVCT_EL0, SYSREG_DAIF, SYSREG_DCZID_EL0, SYSREG_SP_EL0,
    SYSREG_TCR_EL1, SYSREG_TPIDR_EL0, SYSREG_TPIDR_EL1, SYSREG_TPIDRRO_EL0,
};

const JIT_READ_SYSREG_FUNC_INDEX: u32 = 2;

impl WasmExpr {
    pub(super) fn emit_mrs(&mut self, instr: Instr) -> bool {
        if !matches!(
            instr.imm as u16,
            SYSREG_SP_EL0
                | SYSREG_TCR_EL1
                | SYSREG_TPIDR_EL0
                | SYSREG_TPIDR_EL1
                | SYSREG_TPIDRRO_EL0
                | SYSREG_DCZID_EL0
                | SYSREG_CNTPCT_EL0
                | SYSREG_CNTVCT_EL0
                | SYSREG_DAIF
        ) {
            return false;
        }
        self.emit_write_reg_with(instr.rd, true, |this| {
            this.i32_const(instr.imm as i32);
            this.call_func(JIT_READ_SYSREG_FUNC_INDEX);
        });
        true
    }
}
