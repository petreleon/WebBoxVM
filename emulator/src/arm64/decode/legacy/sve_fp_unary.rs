use super::*;

pub(super) fn decode(raw: u32) -> DecodeStep {
    let op = match raw & 0xFF3F_E000 {
        0x041C_A000 => Opcode::SveFpAbs,
        0x041D_A000 => Opcode::SveFpNeg,
        _ => return DecodeStep::Miss,
    };

    let size = 1u8 << (((raw >> 22) & 0x3) as u8);
    if size == 1 {
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
