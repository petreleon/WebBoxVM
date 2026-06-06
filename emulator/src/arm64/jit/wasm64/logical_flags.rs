use super::opcodes::*;
use super::*;
use crate::constants::{PSTATE_NZCV_MASK, PSTATE_N_BIT, PSTATE_Z_BIT};

const LOCAL_RESULT: u32 = 1;

impl WasmExpr {
    pub(super) fn emit_ands_imm(&mut self, instr: crate::arm64::Instr) {
        self.emit_read_reg(instr.rn, instr.sf);
        self.i64_const(instr.imm);
        self.op(OP_I64_AND);
        self.mask_32_if_needed(instr.sf);
        self.local_set(LOCAL_RESULT);

        self.emit_write_reg_with(instr.rd, instr.sf, |this| this.local_get(LOCAL_RESULT));
        self.emit_write_logical_flags(instr.sf);
    }

    fn emit_write_logical_flags(&mut self, sf: bool) {
        self.emit_write_pstate_with(|this| {
            this.emit_read_pstate();
            this.i64_const(!PSTATE_NZCV_MASK);
            this.op(OP_I64_AND);
            this.emit_logical_n_flag(sf);
            this.op(OP_I64_OR);
            this.emit_logical_z_flag();
            this.op(OP_I64_OR);
        });
    }

    fn emit_logical_n_flag(&mut self, sf: bool) {
        self.local_get(LOCAL_RESULT);
        if sf {
            self.i64_const(32);
            self.op(OP_I64_SHR_U);
        }
        self.i64_const(1 << PSTATE_N_BIT);
        self.op(OP_I64_AND);
    }

    fn emit_logical_z_flag(&mut self) {
        self.local_get(LOCAL_RESULT);
        self.op(OP_I64_EQZ);
        self.op(OP_I64_EXTEND_I32_U);
        self.i64_const(PSTATE_Z_BIT as u64);
        self.op(OP_I64_SHL);
    }
}
