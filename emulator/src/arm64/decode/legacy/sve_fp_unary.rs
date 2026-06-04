use super::*;

pub(super) fn decode(raw: u32) -> DecodeStep {
    let op = match raw & 0xFF3F_E000 {
        0x041C_A000 => Opcode::SveFpAbs,
        0x041D_A000 => Opcode::SveFpNeg,
        0x6500_A000 => Opcode::SveFpFrintn,
        0x6503_A000 => Opcode::SveFpFrintz,
        0x6504_A000 => Opcode::SveFpFrinta,
        0x650D_A000 => Opcode::SveFpSqrt,
        _ => return DecodeStep::Miss,
    };

    let size = 1u8 << (((raw >> 22) & 0x3) as u8);
    let supports_half = matches!(op, Opcode::SveFpAbs | Opcode::SveFpNeg);
    if size == 1 || (!supports_half && size == 2) {
        return DecodeStep::Reject;
    }

    DecodeStep::Hit(Instr {
        op,
        rd: (raw & 0x1F) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm: 0,
        imm: 0,
        sf: true,
        cond: ((raw >> 10) & 0x7) as u8,
        size,
    })
}
