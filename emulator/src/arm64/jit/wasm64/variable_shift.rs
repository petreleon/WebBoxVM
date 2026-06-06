use super::opcodes::*;
use super::*;

const LOCAL_VALUE: u32 = 1;
const LOCAL_SHIFT: u32 = 2;

impl WasmExpr {
    pub(super) fn emit_variable_shift(&mut self, instr: crate::arm64::Instr) {
        self.emit_write_reg_with(instr.rd, instr.sf, |this| {
            this.emit_read_reg(instr.rn, instr.sf);
            if !instr.sf && instr.op == Opcode::Asrv {
                this.op(OP_I32_WRAP_I64);
                this.op(OP_I64_EXTEND_I32_S);
            }
            this.emit_shift_amount(instr.rm, instr.sf);
            match instr.op {
                Opcode::Lslv => this.op(OP_I64_SHL),
                Opcode::Lsrv => this.op(OP_I64_SHR_U),
                Opcode::Asrv => this.op(OP_I64_SHR_S),
                Opcode::Rorv if instr.sf => this.op(OP_I64_ROTR),
                Opcode::Rorv => this.emit_rorv32(),
                _ => unreachable!(),
            }
        });
    }

    fn emit_shift_amount(&mut self, rm: u8, sf: bool) {
        self.emit_read_reg(rm, sf);
        self.i64_const(if sf { 63 } else { 31 });
        self.op(OP_I64_AND);
    }

    fn emit_rorv32(&mut self) {
        self.local_set(LOCAL_SHIFT);
        self.local_set(LOCAL_VALUE);
        self.local_get(LOCAL_VALUE);
        self.local_get(LOCAL_SHIFT);
        self.op(OP_I64_SHR_U);
        self.local_get(LOCAL_VALUE);
        self.i64_const(32);
        self.local_get(LOCAL_SHIFT);
        self.op(OP_I64_SUB);
        self.i64_const(31);
        self.op(OP_I64_AND);
        self.op(OP_I64_SHL);
        self.op(OP_I64_OR);
    }
}
