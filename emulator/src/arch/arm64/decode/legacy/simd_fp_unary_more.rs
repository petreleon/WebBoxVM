use super::*;

pub(super) fn decode(raw: u32) -> DecodeStep {
    if let Some(step) = decode_unary(raw, 0x0EA0_F800, Opcode::SimdFpAbsVec) {
        return step;
    }
    if let Some(step) = decode_half_unary(raw, 0x0E39_8800, Opcode::SimdFpFrintnVec) {
        return step;
    }
    if let Some(step) = decode_unary(raw, 0x0E21_8800, Opcode::SimdFpFrintnVec) {
        return step;
    }
    if let Some(step) = decode_unary(raw, 0x2E21_8800, Opcode::SimdFpFrintaVec) {
        return step;
    }
    if let Some(step) = decode_unary(raw, 0x2EA1_F800, Opcode::SimdFpSqrtVec) {
        return step;
    }
    DecodeStep::Miss
}

fn decode_half_unary(raw: u32, base: u32, op: Opcode) -> Option<DecodeStep> {
    if (raw & 0xBFBF_FC00) != base {
        return None;
    }
    Some(DecodeStep::Hit(Instr {
        op,
        rd: (raw & 0x1F) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm: 0,
        imm: 2,
        sf: true,
        cond: 0,
        size: if ((raw >> 30) & 1) != 0 { 16 } else { 8 },
    }))
}

fn decode_unary(raw: u32, base: u32, op: Opcode) -> Option<DecodeStep> {
    if (raw & 0xBFBF_FC00) != base {
        return None;
    }
    let q = ((raw >> 30) & 1) != 0;
    let double = ((raw >> 22) & 1) != 0;
    if double && !q {
        return Some(DecodeStep::Reject);
    }
    Some(DecodeStep::Hit(Instr {
        op,
        rd: (raw & 0x1F) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm: 0,
        imm: if double { 8 } else { 4 },
        sf: true,
        cond: 0,
        size: if q { 16 } else { 8 },
    }))
}
