use super::*;

pub(super) fn decode(raw: u32) -> DecodeStep {
    if let Some(instr) = decode_simd_fp_binary(raw, 0x2E20_D400, Opcode::SimdFpAddp) {
        return DecodeStep::Hit(instr);
    }
    if let Some(instr) = decode_scalar_faddp(raw) {
        return DecodeStep::Hit(instr);
    }
    DecodeStep::Miss
}

fn decode_scalar_faddp(raw: u32) -> Option<Instr> {
    if (raw & 0xFFBF_FC00) != 0x7E30_D800 {
        return None;
    }
    let element_size = if ((raw >> 22) & 1) != 0 { 8 } else { 4 };
    Some(Instr {
        op: Opcode::SimdFpAddp,
        rd: (raw & 0x1F) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm: 0,
        imm: element_size,
        sf: true,
        cond: 0,
        size: element_size as u8,
    })
}
