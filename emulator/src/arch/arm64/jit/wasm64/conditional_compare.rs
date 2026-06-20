use super::opcodes::*;
use super::*;
use crate::constants::{PSTATE_C_BIT, PSTATE_N_BIT, PSTATE_NZCV_MASK, PSTATE_V_BIT, PSTATE_Z_BIT};

const LOCAL_LHS: u32 = 1;
const LOCAL_RHS: u32 = 2;
const LOCAL_RES: u32 = 3;

impl WasmExpr {
    pub(super) fn emit_cond_compare(&mut self, instr: crate::arch::arm64::Instr) {
        self.emit_read_reg(instr.rn, instr.sf);
        self.local_set(LOCAL_LHS);
        if instr.size == 1 {
            self.i64_const(instr.rm as u64);
            self.mask_32_if_needed(instr.sf);
        } else {
            self.emit_read_reg(instr.rm, instr.sf);
        }
        self.local_set(LOCAL_RHS);

        self.emit_write_pstate_with(|this| {
            this.emit_read_pstate();
            this.i64_const(!PSTATE_NZCV_MASK);
            this.op(OP_I64_AND);
            this.emit_condcmp_nzcv(instr.op, instr.sf);
            this.i64_const((instr.imm & 0xf) << PSTATE_V_BIT);
            this.emit_condition(instr.cond);
            this.op(OP_SELECT);
            this.op(OP_I64_OR);
        });
    }

    fn emit_condcmp_nzcv(&mut self, op: Opcode, sf: bool) {
        self.local_get(LOCAL_LHS);
        self.local_get(LOCAL_RHS);
        self.op(if op == Opcode::Ccmn {
            OP_I64_ADD
        } else {
            OP_I64_SUB
        });
        self.mask_32_if_needed(sf);
        self.local_set(LOCAL_RES);

        self.emit_condcmp_n_flag(sf);
        self.emit_condcmp_z_flag();
        self.op(OP_I64_OR);
        self.emit_condcmp_c_flag(op);
        self.op(OP_I64_OR);
        self.emit_condcmp_v_flag(op, sf);
        self.op(OP_I64_OR);
    }

    fn emit_condcmp_n_flag(&mut self, sf: bool) {
        self.local_get(LOCAL_RES);
        if sf {
            self.i64_const(32);
            self.op(OP_I64_SHR_U);
        }
        self.i64_const(1 << PSTATE_N_BIT);
        self.op(OP_I64_AND);
    }

    fn emit_condcmp_z_flag(&mut self) {
        self.local_get(LOCAL_RES);
        self.op(OP_I64_EQZ);
        self.emit_condcmp_i32_flag_bit(PSTATE_Z_BIT);
    }

    fn emit_condcmp_c_flag(&mut self, op: Opcode) {
        self.local_get(LOCAL_LHS);
        self.local_get(LOCAL_RHS);
        self.op(if op == Opcode::Ccmn {
            OP_I64_LT_U
        } else {
            OP_I64_GE_U
        });
        self.emit_condcmp_i32_flag_bit(PSTATE_C_BIT);
    }

    fn emit_condcmp_v_flag(&mut self, op: Opcode, sf: bool) {
        if op == Opcode::Ccmn {
            self.local_get(LOCAL_LHS);
            self.local_get(LOCAL_RES);
            self.op(OP_I64_XOR);
            self.local_get(LOCAL_RHS);
            self.local_get(LOCAL_RES);
        } else {
            self.local_get(LOCAL_LHS);
            self.local_get(LOCAL_RHS);
            self.op(OP_I64_XOR);
            self.local_get(LOCAL_LHS);
            self.local_get(LOCAL_RES);
        }
        self.op(OP_I64_XOR);
        self.op(OP_I64_AND);
        self.i64_const(if sf { 1u64 << 63 } else { 1u64 << 31 });
        self.op(OP_I64_AND);
        self.i64_const(0);
        self.op(OP_I64_NE);
        self.emit_condcmp_i32_flag_bit(PSTATE_V_BIT);
    }

    fn emit_condcmp_i32_flag_bit(&mut self, bit: u32) {
        self.op(OP_I64_EXTEND_I32_U);
        self.i64_const(bit as u64);
        self.op(OP_I64_SHL);
    }
}
