use super::*;

pub(super) fn decode(raw: u32) -> DecodeStep {
    for (base, op) in [
        (0x0E20_F400, Opcode::SimdFpFmaxVec),
        (0x0EA0_F400, Opcode::SimdFpFminVec),
        (0x0E20_C400, Opcode::SimdFpFmaxnmVec),
        (0x0EA0_C400, Opcode::SimdFpFminnmVec),
        (0x2E20_F400, Opcode::SimdFpFmaxp),
        (0x2EA0_F400, Opcode::SimdFpFminp),
        (0x2E20_C400, Opcode::SimdFpFmaxnmp),
        (0x2EA0_C400, Opcode::SimdFpFminnmp),
    ] {
        if let Some(instr) = decode_simd_fp_binary(raw, base, op) {
            return DecodeStep::Hit(instr);
        }
    }
    DecodeStep::Miss
}
