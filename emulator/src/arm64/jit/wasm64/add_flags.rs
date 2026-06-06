use super::opcodes::*;
use super::*;
use crate::constants::{PSTATE_C_BIT, PSTATE_NZCV_MASK, PSTATE_N_BIT, PSTATE_V_BIT, PSTATE_Z_BIT};

const LOCAL_LHS: u32 = 1;
const LOCAL_RHS: u32 = 2;
const LOCAL_RES: u32 = 3;

impl WasmExpr {
    pub(super) fn emit_adds_imm(&mut self, instr: crate::arm64::Instr) {
        self.emit_read_base(instr.rn, instr.sf);
        self.local_set(LOCAL_LHS);
        self.i64_const(instr.imm);
        self.mask_32_if_needed(instr.sf);
        self.local_set(LOCAL_RHS);
        self.emit_add_flags(instr.sf);
        self.emit_add_result(instr.rd, instr.sf, true);
    }

    pub(super) fn emit_adds_reg(&mut self, instr: crate::arm64::Instr) -> bool {
        if instr.cond > 2 || !can_emit_shift(instr.cond, instr.imm, instr.sf) {
            return false;
        }
        self.emit_read_reg(instr.rn, instr.sf);
        self.local_set(LOCAL_LHS);
        self.emit_read_shifted_reg(instr.rm, instr.cond, instr.imm, instr.sf);
        self.mask_32_if_needed(instr.sf);
        self.local_set(LOCAL_RHS);
        self.emit_add_flags(instr.sf);
        self.emit_add_result(instr.rd, instr.sf, false);
        true
    }

    fn emit_add_result(&mut self, rd: u8, sf: bool, allow_sp: bool) {
        if rd == ZERO_REGISTER_INDEX {
            return;
        }
        if allow_sp {
            self.emit_write_reg_sp_with(rd, sf, |this| this.local_get(LOCAL_RES));
        } else {
            self.emit_write_reg_with(rd, sf, |this| this.local_get(LOCAL_RES));
        }
    }

    fn emit_add_flags(&mut self, sf: bool) {
        self.local_get(LOCAL_LHS);
        self.local_get(LOCAL_RHS);
        self.op(OP_I64_ADD);
        self.mask_32_if_needed(sf);
        self.local_set(LOCAL_RES);
        self.emit_write_pstate_with(|this| this.emit_add_nzcv(sf));
    }

    fn emit_add_nzcv(&mut self, sf: bool) {
        self.emit_read_pstate();
        self.i64_const(!PSTATE_NZCV_MASK);
        self.op(OP_I64_AND);
        self.emit_add_n_flag(sf);
        self.op(OP_I64_OR);
        self.emit_add_z_flag();
        self.op(OP_I64_OR);
        self.emit_add_c_flag();
        self.op(OP_I64_OR);
        self.emit_add_v_flag(sf);
        self.op(OP_I64_OR);
    }

    fn emit_add_n_flag(&mut self, sf: bool) {
        self.local_get(LOCAL_RES);
        if sf {
            self.i64_const(32);
            self.op(OP_I64_SHR_U);
        }
        self.i64_const(1 << PSTATE_N_BIT);
        self.op(OP_I64_AND);
    }

    fn emit_add_z_flag(&mut self) {
        self.local_get(LOCAL_RES);
        self.op(OP_I64_EQZ);
        self.emit_add_i32_flag_bit(PSTATE_Z_BIT);
    }

    fn emit_add_c_flag(&mut self) {
        self.local_get(LOCAL_RES);
        self.local_get(LOCAL_LHS);
        self.op(OP_I64_LT_U);
        self.emit_add_i32_flag_bit(PSTATE_C_BIT);
    }

    fn emit_add_v_flag(&mut self, sf: bool) {
        self.local_get(LOCAL_LHS);
        self.local_get(LOCAL_RHS);
        self.op(OP_I64_XOR);
        self.i64_const(width_mask(sf));
        self.op(OP_I64_XOR);
        self.local_get(LOCAL_LHS);
        self.local_get(LOCAL_RES);
        self.op(OP_I64_XOR);
        self.op(OP_I64_AND);
        self.i64_const(sign_mask(sf));
        self.op(OP_I64_AND);
        self.i64_const(0);
        self.op(OP_I64_NE);
        self.emit_add_i32_flag_bit(PSTATE_V_BIT);
    }

    fn emit_add_i32_flag_bit(&mut self, bit: u32) {
        self.op(OP_I64_EXTEND_I32_U);
        self.i64_const(bit as u64);
        self.op(OP_I64_SHL);
    }
}

fn width_mask(sf: bool) -> u64 {
    if sf {
        u64::MAX
    } else {
        u32::MAX as u64
    }
}

fn sign_mask(sf: bool) -> u64 {
    if sf {
        1u64 << 63
    } else {
        1u64 << 31
    }
}
