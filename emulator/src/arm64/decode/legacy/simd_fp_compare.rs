use super::*;

pub(super) fn decode(raw: u32) -> DecodeStep {
    if let Some(step) = decode_compare(raw, 0x2E20_E400, Opcode::SimdFpFcmgeVec, false) {
        return step;
    }
    if let Some(step) = decode_compare(raw, 0x2EA0_E400, Opcode::SimdFpFcmgtVec, false) {
        return step;
    }
    if let Some(step) = decode_compare(raw, 0x0EA0_D800, Opcode::SimdFpFcmeqZero, true) {
        return step;
    }
    if let Some(step) = decode_compare(raw, 0x2EA0_D800, Opcode::SimdFpFcmleZero, true) {
        return step;
    }
    DecodeStep::Miss
}

fn decode_compare(raw: u32, base: u32, op: Opcode, zero: bool) -> Option<DecodeStep> {
    let mask = if zero { 0xBFBF_FC00 } else { 0xBFA0_FC00 };
    if (raw & mask) != base {
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
        rm: if zero { 0 } else { ((raw >> 16) & 0x1F) as u8 },
        imm: if double { 8 } else { 4 },
        sf: true,
        cond: 0,
        size: if q { 16 } else { 8 },
    }))
}
