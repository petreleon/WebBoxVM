use super::*;

impl WasmExpr {
    pub(super) fn emit_simd_instr(&mut self, instr: crate::arm64::Instr) -> bool {
        match instr.op {
            Opcode::SimdLd1
            | Opcode::SimdLd1Multi
            | Opcode::SimdLdp
            | Opcode::SimdLdr
            | Opcode::SimdStr => self.emit_simd_memory_load(instr),
            Opcode::SimdMovi => self.emit_simd_movi(instr),
            op if super::simd_logic::is_logic_reg(op) => self.emit_simd_logic_reg(instr),
            Opcode::SimdDupByte => self.emit_simd_dup_gpr(instr),
            Opcode::SimdFmovDToGpr => self.emit_simd_fmov_d_to_gpr(instr),
            Opcode::SimdCmeqZero => self.emit_simd_cmeq_zero(instr),
            Opcode::SimdCmeqReg => self.emit_simd_cmeq_reg(instr),
            Opcode::SimdCmhsReg | Opcode::SimdCmhiReg => self.emit_simd_unsigned_cmp_reg(instr),
            Opcode::SimdAddhn | Opcode::SimdAddhn2 | Opcode::SimdRaddhn | Opcode::SimdRaddhn2 => {
                self.emit_simd_addhn(instr)
            }
            Opcode::SimdShrn | Opcode::SimdShrn2 | Opcode::SimdRshrn | Opcode::SimdRshrn2 => {
                self.emit_simd_shrn(instr)
            }
            Opcode::SimdAddp => self.emit_simd_addp(instr),
            Opcode::SimdUmaxp => self.emit_simd_umaxp(instr),
            Opcode::SimdUminp => self.emit_simd_uminp(instr),
            _ => false,
        }
    }
}
