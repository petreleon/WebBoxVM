use super::*;

pub(super) fn decode(raw: u32) -> DecodeStep {
    if (raw & 0xFF20_8000) != 0x6400_0000 {
        return DecodeStep::Miss;
    }
    let size = match (raw >> 22) & 0x3 {
        0 => 2,
        1 => 4,
        2 => 8,
        _ => return DecodeStep::Reject,
    };
    DecodeStep::Hit(Instr {
        op: Opcode::SveFpFcmla,
        rd: (raw & 0x1F) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm: ((raw >> 16) & 0x1F) as u8,
        imm: (((raw >> 13) & 0x3) * 90) as u64,
        sf: true,
        cond: ((raw >> 10) & 0x7) as u8,
        size,
    })
}
