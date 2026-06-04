use super::*;

pub(super) fn decode(raw: u32) -> DecodeStep {
    if let Some(instr) = decode_simd_fp_binary(raw, 0x0E20_DC00, Opcode::SimdFpMulx) {
        return DecodeStep::Hit(instr);
    }
    if let Some(instr) = decode_scalar_fmulx(raw) {
        return DecodeStep::Hit(instr);
    }
    DecodeStep::Miss
}

fn decode_scalar_fmulx(raw: u32) -> Option<Instr> {
    if (raw & 0xFF20_FC00) != 0x5E20_DC00 {
        return None;
    }
    let element_size = if ((raw >> 22) & 1) != 0 { 8 } else { 4 };
    Some(Instr {
        op: Opcode::SimdFpMulx,
        rd: (raw & 0x1F) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm: ((raw >> 16) & 0x1F) as u8,
        imm: element_size,
        sf: true,
        cond: 0,
        size: element_size as u8,
    })
}
