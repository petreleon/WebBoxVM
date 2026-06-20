use super::*;

pub(super) fn decode(raw: u32) -> DecodeStep {
    if let Some(op) = sha3_reg(raw) {
        return DecodeStep::Hit(Instr {
            op,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: ((raw >> 16) & 0x1F) as u8,
            imm: 0,
            sf: true,
            cond: 0,
            size: 16,
        });
    }
    if (raw & 0xFFFF_FC00) == 0x5E28_1800 {
        return DecodeStep::Hit(Instr {
            op: Opcode::SimdSha1Su1,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: 0,
            imm: 0,
            sf: true,
            cond: 0,
            size: 16,
        });
    }
    DecodeStep::Miss
}

fn sha3_reg(raw: u32) -> Option<Opcode> {
    match raw & 0xFFE0_FC00 {
        0x5E00_0000 => Some(Opcode::SimdSha1C),
        0x5E00_1000 => Some(Opcode::SimdSha1P),
        0x5E00_2000 => Some(Opcode::SimdSha1M),
        0x5E00_3000 => Some(Opcode::SimdSha1Su0),
        0x5E00_4000 => Some(Opcode::SimdSha256H),
        0x5E00_5000 => Some(Opcode::SimdSha256H2),
        0x5E00_6000 => Some(Opcode::SimdSha256Su1),
        _ => None,
    }
}
