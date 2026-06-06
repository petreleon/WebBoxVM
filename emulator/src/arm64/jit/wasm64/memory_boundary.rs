use super::*;
use crate::arm64::{Instr, Opcode};

impl WasmExpr {
    pub(super) fn emit_memory_boundary(&mut self, instr: Instr) -> Option<bool> {
        match instr.op {
            Opcode::Str => Some(self.emit_memory_store(instr)),
            Opcode::Stp => Some(self.emit_memory_pair_store(instr)),
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
