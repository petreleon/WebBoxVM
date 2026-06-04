use super::*;

pub(super) fn decode(raw: u32) -> DecodeStep {
    if let Some(instr) = decode_scalar_pairwise_minmax(raw) {
        return DecodeStep::Hit(instr);
    }
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

fn decode_scalar_pairwise_minmax(raw: u32) -> Option<Instr> {
    let op = match raw & 0xFFBF_FC00 {
        0x7E30_F800 => Opcode::SimdFpFmaxp,
        0x7EB0_F800 => Opcode::SimdFpFminp,
        0x7E30_C800 => Opcode::SimdFpFmaxnmp,
        0x7EB0_C800 => Opcode::SimdFpFminnmp,
        _ => return None,
    };
    let element_size = if ((raw >> 22) & 1) != 0 { 8 } else { 4 };
    Some(Instr {
        op,
        rd: (raw & 0x1F) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm: 0,
        imm: element_size,
        sf: true,
        cond: 0,
        size: element_size as u8,
    })
}
