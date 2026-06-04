use super::*;

pub(super) fn decode(raw: u32) -> DecodeStep {
    for (base, op) in [
        (0x0E20_F400, Opcode::SimdFpFmaxVec),
        (0x0EA0_F400, Opcode::SimdFpFminVec),
        (0x0E20_C400, Opcode::SimdFpFmaxnmVec),
        (0x0EA0_C400, Opcode::SimdFpFminnmVec),
    ] {
        if let Some(instr) = decode_simd_fp_binary(raw, base, op) {
            return DecodeStep::Hit(instr);
        }
    }
    DecodeStep::Miss
}
