use super::opcodes::*;
use super::*;

impl WasmExpr {
    pub(super) fn emit_extract(&mut self, instr: crate::arch::arm64::Instr) {
        let width = if instr.sf { 64 } else { 32 };
        let lsb = instr.imm & (width - 1);

        self.emit_write_reg_with(instr.rd, instr.sf, |this| {
            this.emit_read_reg(instr.rm, instr.sf);
            if lsb == 0 {
                return;
            }
            this.i64_const(lsb);
            this.op(OP_I64_SHR_U);
            this.emit_read_reg(instr.rn, instr.sf);
            this.i64_const(width - lsb);
            this.op(OP_I64_SHL);
            this.op(OP_I64_OR);
        });
    }
}
