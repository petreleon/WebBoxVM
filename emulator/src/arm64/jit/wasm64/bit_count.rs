use super::opcodes::*;
use super::*;

impl WasmExpr {
    pub(super) fn emit_clz(&mut self, instr: crate::arm64::Instr) {
        self.emit_write_reg_with(instr.rd, instr.sf, |this| {
            this.emit_read_reg(instr.rn, instr.sf);
            if instr.sf {
                this.op(OP_I64_CLZ);
            } else {
                this.op(OP_I32_WRAP_I64);
                this.op(OP_I32_CLZ);
                this.op(OP_I64_EXTEND_I32_U);
            }
        });
    }
}
