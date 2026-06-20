use super::*;

pub(super) fn decode(raw: u32) -> Option<Instr> {
    let op = match raw & 0xFFFF_FC00 {
        0xDAC1_0000 => Opcode::Pacia,
        0xDAC1_0400 => Opcode::Pacib,
        0xDAC1_0800 => Opcode::Pacda,
        0xDAC1_0C00 => Opcode::Pacdb,
        0xDAC1_1000 => Opcode::Autia,
        0xDAC1_1400 => Opcode::Autib,
        0xDAC1_1800 => Opcode::Autda,
        0xDAC1_1C00 => Opcode::Autdb,
        _ => return decode_fixed_rn(raw),
    };
    Some(instr(raw, op, ((raw >> 5) & 0x1F) as u8))
}

fn decode_fixed_rn(raw: u32) -> Option<Instr> {
    let op = match raw & 0xFFFF_FFE0 {
        0xDAC1_23E0 => Opcode::Paciza,
        0xDAC1_27E0 => Opcode::Pacizb,
        0xDAC1_2BE0 => Opcode::Pacdza,
        0xDAC1_2FE0 => Opcode::Pacdzb,
        0xDAC1_33E0 => Opcode::Autiza,
        0xDAC1_37E0 => Opcode::Autizb,
        0xDAC1_3BE0 => Opcode::Autdza,
        0xDAC1_3FE0 => Opcode::Autdzb,
        0xDAC1_43E0 => Opcode::Xpaci,
        0xDAC1_47E0 => Opcode::Xpacd,
        _ => return None,
    };
    Some(instr(raw, op, 0))
}

fn instr(raw: u32, op: Opcode, rn: u8) -> Instr {
    Instr {
        op,
        rd: (raw & 0x1F) as u8,
        rn,
        rm: 0,
        imm: 0,
        sf: true,
        cond: 0,
        size: 0,
    }
}
