use super::*;

pub(super) fn decode(raw: u32) -> DecodeStep {
    let op = match raw & 0xFF3F_E000 {
        0x6500_8000 => Opcode::SveFpAdd,
        0x6501_8000 => Opcode::SveFpSub,
        0x6502_8000 => Opcode::SveFpMul,
        0x6503_8000 => Opcode::SveFpSubr,
        _ => return DecodeStep::Miss,
    };
    let size = 1u8 << (((raw >> 22) & 0x3) as u8);
    if size == 1 {
        return DecodeStep::Reject;
    }
    let rd = (raw & 0x1F) as u8;
    DecodeStep::Hit(Instr {
        op,
        rd,
        rn: rd,
        rm: ((raw >> 5) & 0x1F) as u8,
        imm: 0,
        sf: true,
        cond: ((raw >> 10) & 0x7) as u8,
        size,
    })
}
