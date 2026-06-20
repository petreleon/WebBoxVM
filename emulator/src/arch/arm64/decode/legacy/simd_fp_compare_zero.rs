use super::*;

pub(super) fn decode(raw: u32) -> DecodeStep {
    if (raw & 0xBFBF_FC00) != 0x0EA0_E800 {
        return DecodeStep::Miss;
    }
    let q = ((raw >> 30) & 1) != 0;
    let double = ((raw >> 22) & 1) != 0;
    if double && !q {
        return DecodeStep::Reject;
    }
    DecodeStep::Hit(Instr {
        op: Opcode::SimdFpFcmltZero,
        rd: (raw & 0x1F) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm: 0,
        imm: if double { 8 } else { 4 },
        sf: true,
        cond: 0,
        size: if q { 16 } else { 8 },
    })
}
