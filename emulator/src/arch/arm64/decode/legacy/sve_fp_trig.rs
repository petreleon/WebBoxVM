use super::*;

pub(super) fn decode(raw: u32) -> DecodeStep {
    let op = match raw & 0xFF20_FC00 {
        0x6500_0C00 => Opcode::SveFpFtsmul,
        0x0420_B000 => Opcode::SveFpFtssel,
        _ => return DecodeStep::Miss,
    };

    let size_bits = ((raw >> 22) & 0x3) as u8;
    if size_bits == 0 {
        return DecodeStep::Reject;
    }

    DecodeStep::Hit(Instr {
        op,
        rd: (raw & 0x1F) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm: ((raw >> 16) & 0x1F) as u8,
        imm: 0,
        sf: true,
        cond: 0xFF,
        size: 1u8 << size_bits,
    })
}
