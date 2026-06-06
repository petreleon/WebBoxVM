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

    fn emit_signed_word(&mut self, reg: u8) {
        self.emit_read_reg(reg, false);
        self.op(OP_I32_WRAP_I64);
        self.op(OP_I64_EXTEND_I32_S);
    }
}
