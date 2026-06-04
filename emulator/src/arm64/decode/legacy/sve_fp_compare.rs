use super::*;

pub(super) fn decode(raw: u32) -> DecodeStep {
    let op = match raw & 0xFF20_E010 {
        0x6500_C010 => Opcode::SveFpFacge,
        0x6500_E010 => Opcode::SveFpFacgt,
        _ => return DecodeStep::Miss,
    };
    let size = 1u8 << (((raw >> 22) & 0x3) as u8);
    if size == 1 {
        return DecodeStep::Reject;
    }
    DecodeStep::Hit(Instr {
        op,
        rd: (raw & 0xF) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm: ((raw >> 16) & 0x1F) as u8,
        imm: 0,
        sf: true,
        cond: ((raw >> 10) & 0x7) as u8,
        size,
    })
}
