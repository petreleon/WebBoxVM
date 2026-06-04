use super::*;

pub(super) fn decode(raw: u32) -> DecodeStep {
    let op = match raw & 0xFF20_FC00 {
        0x0520_6000 => Opcode::SveZip1,
        0x0520_6400 => Opcode::SveZip2,
        _ => return DecodeStep::Miss,
    };

    DecodeStep::Hit(Instr {
        op,
        rd: (raw & 0x1F) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm: ((raw >> 16) & 0x1F) as u8,
        imm: 0,
        sf: true,
        cond: 0xFF,
        size: 1u8 << (((raw >> 22) & 0x3) as u8),
    })
}
