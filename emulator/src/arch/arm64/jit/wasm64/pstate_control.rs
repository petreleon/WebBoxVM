use super::opcodes::*;
use super::*;
use crate::arch::arm64::{Instr, Opcode};
use crate::constants::PSTATE_I_BIT;

const PSTATE_I_MASK: u64 = 1 << PSTATE_I_BIT;

impl WasmExpr {
    pub(super) fn emit_daif_imm(&mut self, instr: Instr) {
        if (instr.imm & 2) == 0 {
            return;
        }

        self.emit_write_pstate_with(|this| {
            this.emit_read_pstate();
            match instr.op {
                Opcode::DaifSet => {
                    this.i64_const(PSTATE_I_MASK);
                    this.op(OP_I64_OR);
                }
                Opcode::DaifClr => {
                    this.i64_const(!PSTATE_I_MASK);
                    this.op(OP_I64_AND);
                }
                _ => unreachable!(),
            }
        });
    }
}
