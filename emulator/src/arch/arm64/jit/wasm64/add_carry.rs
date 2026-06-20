use super::opcodes::*;
use super::*;
use crate::constants::{PSTATE_C_BIT, PSTATE_N_BIT, PSTATE_NZCV_MASK, PSTATE_V_BIT, PSTATE_Z_BIT};

const LOCAL_LHS: u32 = 1;
const LOCAL_RHS: u32 = 2;
const LOCAL_RES: u32 = 3;
const LOCAL_CARRY: u32 = 4;

impl WasmExpr {
    pub(super) fn emit_addsub_carry(&mut self, instr: crate::arch::arm64::Instr) {
        self.emit_read_reg(instr.rn, instr.sf);
        self.local_set(LOCAL_LHS);
        self.emit_carry_rhs(instr);
        self.local_set(LOCAL_RHS);
        self.emit_carry_input();
        self.local_set(LOCAL_CARRY);
        self.emit_carry_result(instr.sf);
        if matches!(instr.op, Opcode::Adcs | Opcode::Sbcs) {
            self.emit_carry_flags(instr.sf);
        }
        self.emit_write_reg_with(instr.rd, instr.sf, |this| this.local_get(LOCAL_RES));
    }

    fn emit_carry_rhs(&mut self, instr: crate::arch::arm64::Instr) {
        self.emit_read_reg(instr.rm, instr.sf);
        if matches!(instr.op, Opcode::Sbc | Opcode::Sbcs) {
            self.i64_const(width_mask(instr.sf));
            self.op(OP_I64_XOR);
            self.mask_32_if_needed(instr.sf);
        }
    }

    fn emit_carry_input(&mut self) {
        self.emit_read_pstate();
        self.i64_const(PSTATE_C_BIT as u64);
        self.op(OP_I64_SHR_U);
        self.i64_const(1);
        self.op(OP_I64_AND);
    }

    fn emit_carry_result(&mut self, sf: bool) {
        self.local_get(LOCAL_LHS);
        self.local_get(LOCAL_RHS);
        self.op(OP_I64_ADD);
        self.local_get(LOCAL_CARRY);
        self.op(OP_I64_ADD);
        self.mask_32_if_needed(sf);
        self.local_set(LOCAL_RES);
    }

    fn emit_carry_flags(&mut self, sf: bool) {
        self.emit_write_pstate_with(|this| {
            this.emit_read_pstate();
            this.i64_const(!PSTATE_NZCV_MASK);
            this.op(OP_I64_AND);
            this.emit_carry_n_flag(sf);
            this.op(OP_I64_OR);
            this.emit_carry_z_flag();
            this.op(OP_I64_OR);
            this.emit_carry_c_flag(sf);
            this.op(OP_I64_OR);
            this.emit_carry_v_flag(sf);
            this.op(OP_I64_OR);
        });
    }

    fn emit_carry_n_flag(&mut self, sf: bool) {
        self.local_get(LOCAL_RES);
        if sf {
            self.i64_const(32);
            self.op(OP_I64_SHR_U);
        }
        self.i64_const(1 << PSTATE_N_BIT);
        self.op(OP_I64_AND);
    }

    fn emit_carry_z_flag(&mut self) {
        self.local_get(LOCAL_RES);
        self.op(OP_I64_EQZ);
        self.emit_carry_i32_flag_bit(PSTATE_Z_BIT);
    }

    fn emit_carry_c_flag(&mut self, sf: bool) {
        if sf {
            self.emit_carry_out_64();
        } else {
            self.emit_carry_out_32();
        }
        self.emit_carry_i32_flag_bit(PSTATE_C_BIT);
    }

    fn emit_carry_out_64(&mut self) {
        self.local_get(LOCAL_LHS);
        self.local_get(LOCAL_RHS);
        self.op(OP_I64_ADD);
        self.local_get(LOCAL_LHS);
        self.op(OP_I64_LT_U);
        self.local_get(LOCAL_RES);
        self.local_get(LOCAL_LHS);
        self.local_get(LOCAL_RHS);
        self.op(OP_I64_ADD);
        self.op(OP_I64_LT_U);
        self.op(OP_I32_OR);
    }

    fn emit_carry_out_32(&mut self) {
        self.i64_const(u32::MAX as u64);
        self.local_get(LOCAL_LHS);
        self.local_get(LOCAL_RHS);
        self.op(OP_I64_ADD);
        self.local_get(LOCAL_CARRY);
        self.op(OP_I64_ADD);
        self.op(OP_I64_LT_U);
    }

    fn emit_carry_v_flag(&mut self, sf: bool) {
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
        self.emit_carry_i32_flag_bit(PSTATE_V_BIT);
    }

    fn emit_carry_i32_flag_bit(&mut self, bit: u32) {
        self.op(OP_I64_EXTEND_I32_U);
        self.i64_const(bit as u64);
        self.op(OP_I64_SHL);
    }
}

fn width_mask(sf: bool) -> u64 {
    if sf { u64::MAX } else { u32::MAX as u64 }
}

fn sign_mask(sf: bool) -> u64 {
    if sf { 1u64 << 63 } else { 1u64 << 31 }
}
