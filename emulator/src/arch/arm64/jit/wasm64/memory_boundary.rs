use super::*;
use crate::arch::arm64::{Instr, Opcode};

impl WasmExpr {
    pub(super) fn emit_memory_boundary(&mut self, instr: Instr) -> Option<bool> {
        match instr.op {
            Opcode::Ldxr => Some(self.emit_exclusive_load(instr)),
            Opcode::Ldxp => Some(self.emit_exclusive_pair_load(instr)),
            Opcode::Str => Some(self.emit_memory_store(instr)),
            Opcode::Stxr => Some(self.emit_exclusive_store(instr)),
            Opcode::Stp => Some(self.emit_memory_pair_store(instr)),
            Opcode::Stxp => Some(self.emit_exclusive_pair_store(instr)),
            Opcode::SimdStp => Some(self.emit_simd_stp(instr)),
            Opcode::Ldp | Opcode::Ldpsw => Some(self.emit_memory_pair_load(instr)),
            Opcode::DcZva => {
                self.emit_dc_zva(instr);
                Some(true)
            }
            _ => None,
        }
    }
}
