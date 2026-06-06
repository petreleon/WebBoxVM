use super::opcodes::*;
use super::*;

impl WasmExpr {
    pub(super) fn emit_cond_select(&mut self, instr: crate::arm64::Instr) {
        self.emit_write_reg_with(instr.rd, instr.sf, |this| {
            this.emit_read_reg(instr.rn, instr.sf);
            this.emit_false_operand(instr);
            this.emit_condition(instr.cond);
            this.op(OP_SELECT);
        });
    }

    fn emit_false_operand(&mut self, instr: crate::arm64::Instr) {
        self.emit_read_reg(instr.rm, instr.sf);
        match instr.op {
            Opcode::Csel => {}
            Opcode::Csinc => {
                self.i64_const(1);
                self.op(OP_I64_ADD);
            }
            Opcode::Csinv => {
                self.i64_const(u64::MAX);
                self.op(OP_I64_XOR);
            }
            Opcode::Csneg => {
                self.i64_const(u64::MAX);
                self.op(OP_I64_XOR);
                self.i64_const(1);
                self.op(OP_I64_ADD);
            }
            _ => unreachable!(),
        }
    }
}
