use super::opcodes::*;
use super::*;
use crate::constants::{PSTATE_C_BIT, PSTATE_N_BIT, PSTATE_NZCV_MASK, PSTATE_V_BIT, PSTATE_Z_BIT};

const LOCAL_LHS: u32 = 1;
const LOCAL_RHS: u32 = 2;
const LOCAL_RES: u32 = 3;

impl WasmExpr {
    pub(super) fn emit_cmp_imm(&mut self, instr: crate::arch::arm64::Instr) {
        self.emit_read_base(instr.rn, instr.sf);
        self.local_set(LOCAL_LHS);
        self.i64_const(instr.imm);
        self.mask_32_if_needed(instr.sf);
        self.local_set(LOCAL_RHS);
        self.emit_sub_flags(instr.sf);
    }

    pub(super) fn emit_subs_imm(&mut self, instr: crate::arch::arm64::Instr) {
        self.emit_read_base(instr.rn, instr.sf);
        self.local_set(LOCAL_LHS);
        self.i64_const(instr.imm);
        self.mask_32_if_needed(instr.sf);
        self.local_set(LOCAL_RHS);
        self.emit_sub_flags(instr.sf);
        if instr.rd != ZERO_REGISTER_INDEX {
            self.emit_write_reg_sp_with(instr.rd, instr.sf, |this| this.local_get(LOCAL_RES));
        }
    }

    pub(super) fn emit_cmp_reg(&mut self, instr: crate::arch::arm64::Instr) -> bool {
        if (instr.cond & 0x8) != 0 {
            return self.emit_cmp_ext(instr);
        }
        if !can_emit_shift(instr.cond, instr.imm, instr.sf) {
            return false;
        }
        self.emit_read_reg(instr.rn, instr.sf);
        self.local_set(LOCAL_LHS);
        self.emit_read_shifted_reg(instr.rm, instr.cond, instr.imm, instr.sf);
        self.mask_32_if_needed(instr.sf);
        self.local_set(LOCAL_RHS);
        self.emit_sub_flags(instr.sf);
        true
    }

    pub(super) fn emit_subs_reg(&mut self, instr: crate::arch::arm64::Instr) -> bool {
        if (instr.cond & 0x8) != 0 || !can_emit_shift(instr.cond, instr.imm, instr.sf) {
            return false;
        }
        self.emit_read_reg(instr.rn, instr.sf);
        self.local_set(LOCAL_LHS);
        self.emit_read_shifted_reg(instr.rm, instr.cond, instr.imm, instr.sf);
        self.mask_32_if_needed(instr.sf);
        self.local_set(LOCAL_RHS);
        self.emit_sub_flags(instr.sf);
        self.emit_write_reg_with(instr.rd, instr.sf, |this| this.local_get(LOCAL_RES));
        true
    }

    fn emit_cmp_ext(&mut self, instr: crate::arch::arm64::Instr) -> bool {
        let option = instr.cond & 0x7;
        if instr.imm > 4 {
            return false;
        }
        self.emit_read_base(instr.rn, instr.sf);
        self.local_set(LOCAL_LHS);
        self.emit_extended_reg(instr.rm, option, instr.imm, instr.sf);
        self.local_set(LOCAL_RHS);
        self.emit_sub_flags(instr.sf);
        true
    }

    fn emit_sub_flags(&mut self, sf: bool) {
        self.local_get(LOCAL_LHS);
        self.local_get(LOCAL_RHS);
        self.op(OP_I64_SUB);
        self.mask_32_if_needed(sf);
        self.local_set(LOCAL_RES);

        self.emit_write_pstate_with(|this| {
            this.emit_read_pstate();
            this.i64_const(!PSTATE_NZCV_MASK);
            this.op(OP_I64_AND);
            this.emit_n_flag(sf);
            this.op(OP_I64_OR);
            this.emit_z_flag();
            this.op(OP_I64_OR);
            this.emit_c_flag();
            this.op(OP_I64_OR);
            this.emit_v_flag(sf);
            this.op(OP_I64_OR);
        });
    }

    fn emit_n_flag(&mut self, sf: bool) {
        self.local_get(LOCAL_RES);
        if sf {
            self.i64_const(32);
            self.op(OP_I64_SHR_U);
        }
        self.i64_const(1 << PSTATE_N_BIT);
        self.op(OP_I64_AND);
    }

    fn emit_z_flag(&mut self) {
        self.local_get(LOCAL_RES);
        self.op(OP_I64_EQZ);
        self.emit_i32_flag_bit(PSTATE_Z_BIT);
    }

    fn emit_c_flag(&mut self) {
        self.local_get(LOCAL_LHS);
        self.local_get(LOCAL_RHS);
        self.op(OP_I64_GE_U);
        self.emit_i32_flag_bit(PSTATE_C_BIT);
    }

    fn emit_v_flag(&mut self, sf: bool) {
        let sign = if sf { 1u64 << 63 } else { 1u64 << 31 };
        self.local_get(LOCAL_LHS);
        self.local_get(LOCAL_RHS);
        self.op(OP_I64_XOR);
        self.local_get(LOCAL_LHS);
        self.local_get(LOCAL_RES);
        self.op(OP_I64_XOR);
        self.op(OP_I64_AND);
        self.i64_const(sign);
        self.op(OP_I64_AND);
        self.i64_const(0);
        self.op(OP_I64_NE);
        self.emit_i32_flag_bit(PSTATE_V_BIT);
    }

    fn emit_i32_flag_bit(&mut self, bit: u32) {
        self.op(OP_I64_EXTEND_I32_U);
        self.i64_const(bit as u64);
        self.op(OP_I64_SHL);
    }
}
