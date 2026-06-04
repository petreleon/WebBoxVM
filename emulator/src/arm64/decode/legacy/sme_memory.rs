use super::*;

pub(super) fn decode(raw: u32) -> DecodeStep {
    let op = match raw & 0xFFFF_9C10 {
        0xE100_0000 => Opcode::SmeLdrZa,
        0xE120_0000 => Opcode::SmeStrZa,
        _ => return DecodeStep::Miss,
    };
    let rv = ((raw >> 13) & 0x3) as u8;
    DecodeStep::Hit(Instr {
        op,
        rd: 12 + rv,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm: 0xFF,
        imm: (raw & 0xF) as u64,
        sf: true,
        cond: rv,
        size: 1,
    })
}
