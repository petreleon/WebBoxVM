use super::opcodes::*;
use super::*;
use crate::arm64::Instr;

impl WasmExpr {
    pub(super) fn emit_simd_dup_gpr(&mut self, instr: Instr) -> bool {
        if instr.cond != 4 || instr.size != 16 {
            return false;
        }
        self.emit_dup_s_half(instr.rd, instr.rn, false);
        self.emit_dup_s_half(instr.rd, instr.rn, true);
        true
    }

    fn emit_dup_s_half(&mut self, rd: u8, rn: u8, high: bool) {
        self.emit_write_simd_half_with(rd, high, |this| {
            this.emit_read_reg(rn, false);
            this.emit_read_reg(rn, false);
            this.i64_const(32);
            this.op(OP_I64_SHL);
            this.op(OP_I64_OR);
        });
    }
}
