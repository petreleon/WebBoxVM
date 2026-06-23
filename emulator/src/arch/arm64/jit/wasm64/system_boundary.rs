use super::*;
use crate::arch::arm64::{Instr, Opcode};
use crate::constants::SYSREG_SP_EL0;

impl WasmExpr {
    pub(super) fn emit_system_boundary(&mut self, instr: Instr) -> Option<bool> {
        match instr.op {
            Opcode::Msr if instr.imm as u16 == SYSREG_SP_EL0 => Some(self.emit_msr(instr)),
            _ => None,
        }
    }
}
