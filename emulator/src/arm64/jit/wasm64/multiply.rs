use super::opcodes::*;
use super::*;

impl WasmExpr {
    pub(super) fn emit_madd_msub(&mut self, instr: crate::arm64::Instr) -> bool {
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

    fn emit_mul_operands(&mut self, instr: crate::arm64::Instr) {
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

    pub(super) fn emit_udiv(&mut self, instr: crate::arm64::Instr) {
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
