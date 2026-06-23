use super::opcodes::{OP_I64_AND, OP_I64_OR};
use super::*;
use crate::arch::arm64::Instr;
use crate::constants::{
    DCZID_EL0_VAL, PSTATE_DAIF_MASK, PSTATE_EL_MASK, SYSREG_CNTPCT_EL0, SYSREG_CNTVCT_EL0,
    SYSREG_CURRENTEL, SYSREG_DAIF, SYSREG_DCZID_EL0, SYSREG_SP_EL0, SYSREG_SPSR_EL1,
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
                | SYSREG_SPSR_EL1
                | SYSREG_CURRENTEL
                | SYSREG_DCZID_EL0
                | SYSREG_CNTPCT_EL0
                | SYSREG_CNTVCT_EL0
                | SYSREG_DAIF
        ) {
            return false;
        }
        match instr.imm as u16 {
            SYSREG_DAIF => self.emit_write_pstate_sysreg(instr.rd, PSTATE_DAIF_MASK),
            SYSREG_CURRENTEL => self.emit_write_pstate_sysreg(instr.rd, PSTATE_EL_MASK),
            SYSREG_DCZID_EL0 => self.emit_write_reg_with(instr.rd, true, |this| {
                this.i64_const(DCZID_EL0_VAL);
            }),
            _ => self.emit_write_reg_with(instr.rd, true, |this| {
                this.i32_const(instr.imm as i32);
                this.call_func(JIT_READ_SYSREG_FUNC_INDEX);
            }),
        }
        true
    }

    pub(super) fn emit_msr(&mut self, instr: Instr) -> bool {
        if instr.imm as u16 != SYSREG_DAIF {
            return false;
        }
        self.emit_write_pstate_with(|this| {
            this.emit_read_pstate();
            this.i64_const(!PSTATE_DAIF_MASK);
            this.op(OP_I64_AND);
            this.emit_read_reg(instr.rd, true);
            this.i64_const(PSTATE_DAIF_MASK);
            this.op(OP_I64_AND);
            this.op(OP_I64_OR);
        });
        true
    }

    fn emit_write_pstate_sysreg(&mut self, rd: u8, mask: u64) {
        self.emit_write_reg_with(rd, true, |this| {
            this.emit_read_pstate();
            this.i64_const(mask);
            this.op(OP_I64_AND);
        });
    }
}
