use super::*;

pub(super) fn decode(raw: u32) -> DecodeStep {
    if let Some(instr) = decode_sve_tbl(raw) {
        return DecodeStep::Hit(instr);
    }

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

fn decode_sve_tbl(raw: u32) -> Option<Instr> {
    let table_count = match raw & 0xFF20_FC00 {
        0x0520_3000 => 1,
        0x0520_2800 => 2,
        _ => return None,
    };

    Some(Instr {
        op: Opcode::SveTbl,
        rd: (raw & 0x1F) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm: ((raw >> 16) & 0x1F) as u8,
        imm: table_count,
        sf: true,
        cond: 0xFF,
        size: 1u8 << (((raw >> 22) & 0x3) as u8),
    })
}
