use super::opcodes::*;
use super::*;

impl WasmExpr {
    pub(super) fn emit_madd_msub(&mut self, instr: crate::arch::arm64::Instr) -> bool {
        if instr.size > 2 {
            return false;
        }
        let sub = instr.op == Opcode::Msub;
        self.emit_write_reg_with(instr.rd, instr.sf, |this| {
            this.emit_read_reg(instr.cond, instr.sf);
            this.emit_mul_operands(instr);
            this.op(if sub { OP_I64_SUB } else { OP_I64_ADD });
        });
        true
    }

    fn emit_mul_operands(&mut self, instr: crate::arch::arm64::Instr) {
        match instr.size {
            0 => {
                self.emit_read_reg(instr.rn, instr.sf);
                self.emit_read_reg(instr.rm, instr.sf);
            }
            1 => {
                self.emit_read_reg(instr.rn, false);
                self.emit_read_reg(instr.rm, false);
            }
            2 => {
                self.emit_signed_word(instr.rn);
                self.emit_signed_word(instr.rm);
            }
            _ => unreachable!(),
        }
        self.op(OP_I64_MUL);
    }

    pub(super) fn emit_umulh(&mut self, instr: crate::arch::arm64::Instr) -> bool {
        if !instr.sf {
            return false;
        }
        self.emit_split_u64(instr.rn, LOCAL_UMULH_X0, LOCAL_UMULH_X1);
        self.emit_split_u64(instr.rm, LOCAL_UMULH_Y0, LOCAL_UMULH_Y1);
        self.emit_mul_locals(LOCAL_UMULH_X0, LOCAL_UMULH_Y0);
        self.local_set(LOCAL_UMULH_T);
        self.emit_mul_locals(LOCAL_UMULH_X1, LOCAL_UMULH_Y0);
        self.local_get(LOCAL_UMULH_T);
        self.emit_ushr_32();
        self.op(OP_I64_ADD);
        self.local_set(LOCAL_UMULH_T);
        self.local_get(LOCAL_UMULH_T);
        self.emit_low_32();
        self.local_set(LOCAL_UMULH_W1);
        self.local_get(LOCAL_UMULH_T);
        self.emit_ushr_32();
        self.local_set(LOCAL_UMULH_W2);
        self.emit_mul_locals(LOCAL_UMULH_X0, LOCAL_UMULH_Y1);
        self.local_get(LOCAL_UMULH_W1);
        self.op(OP_I64_ADD);
        self.local_set(LOCAL_UMULH_T);
        self.emit_write_reg_with(instr.rd, true, |this| {
            this.emit_mul_locals(LOCAL_UMULH_X1, LOCAL_UMULH_Y1);
            this.local_get(LOCAL_UMULH_W2);
            this.op(OP_I64_ADD);
            this.local_get(LOCAL_UMULH_T);
            this.emit_ushr_32();
            this.op(OP_I64_ADD);
        });
        true
    }

    fn emit_split_u64(&mut self, reg: u8, lo: u32, hi: u32) {
        self.emit_read_reg(reg, true);
        self.emit_low_32();
        self.local_set(lo);
        self.emit_read_reg(reg, true);
        self.emit_ushr_32();
        self.local_set(hi);
    }

    fn emit_mul_locals(&mut self, lhs: u32, rhs: u32) {
        self.local_get(lhs);
        self.local_get(rhs);
        self.op(OP_I64_MUL);
    }

    fn emit_low_32(&mut self) {
        self.i64_const(u32::MAX as u64);
        self.op(OP_I64_AND);
    }

    fn emit_ushr_32(&mut self) {
        self.i64_const(32);
        self.op(OP_I64_SHR_U);
    }

    pub(super) fn emit_udiv(&mut self, instr: crate::arch::arm64::Instr) {
        self.emit_read_reg(instr.rm, instr.sf);
        self.local_set(LOCAL_DIVISOR);
        self.emit_write_reg_with(instr.rd, instr.sf, |this| {
            this.emit_read_reg(instr.rn, instr.sf);
            this.emit_safe_divisor();
            this.op(OP_I64_DIV_U);
            this.emit_zero_when_divisor_zero();
        });
    }

    fn emit_safe_divisor(&mut self) {
        self.local_get(LOCAL_DIVISOR);
        self.i64_const(1);
        self.emit_divisor_nonzero();
        self.op(OP_SELECT);
    }

    fn emit_zero_when_divisor_zero(&mut self) {
        self.i64_const(0);
        self.emit_divisor_nonzero();
        self.op(OP_SELECT);
    }

    fn emit_divisor_nonzero(&mut self) {
        self.local_get(LOCAL_DIVISOR);
        self.i64_const(0);
        self.op(OP_I64_NE);
    }

    fn emit_signed_word(&mut self, reg: u8) {
        self.emit_read_reg(reg, false);
        self.op(OP_I32_WRAP_I64);
        self.op(OP_I64_EXTEND_I32_S);
    }
}

const LOCAL_DIVISOR: u32 = 4;
const LOCAL_UMULH_X0: u32 = 1;
const LOCAL_UMULH_X1: u32 = 2;
const LOCAL_UMULH_Y0: u32 = 3;
const LOCAL_UMULH_Y1: u32 = 4;
const LOCAL_UMULH_T: u32 = 5;
const LOCAL_UMULH_W1: u32 = 6;
const LOCAL_UMULH_W2: u32 = 7;
