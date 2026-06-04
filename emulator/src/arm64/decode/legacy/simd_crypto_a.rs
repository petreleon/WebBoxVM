use super::*;

pub(super) fn decode(raw: u32) -> DecodeStep {
    if (raw & 0xBF20_FC00) == 0x0E20_E000 {
        let size_bits = (raw >> 22) & 0x3;
        if matches!(size_bits, 1 | 2) {
            return DecodeStep::Reject;
        }
        return DecodeStep::Hit(Instr {
            op: Opcode::SimdPmull,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: ((raw >> 16) & 0x1F) as u8,
            imm: 1u64 << size_bits,
            sf: ((raw >> 30) & 1) != 0,
            cond: 0,
            size: 16,
        });
    }
    if (raw & 0xFFFF_FC00) == 0x5E28_0800 {
        return DecodeStep::Hit(Instr {
            op: Opcode::SimdSha1h,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: 0,
            imm: 0,
            sf: true,
            cond: 0,
            size: 4,
        });
    }
    if (raw & 0xFFFF_FC00) == 0x5E28_2800 {
        return DecodeStep::Hit(Instr {
            op: Opcode::SimdSha256Su0,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: 0,
            imm: 0,
            sf: true,
            cond: 0,
            size: 16,
        });
    }
    if (raw & 0xFFFF_FC00) == 0xCEC0_8000 {
        return DecodeStep::Hit(Instr {
            op: Opcode::SimdSha512Su0,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: 0,
            imm: 0,
            sf: true,
            cond: 0,
            size: 16,
        });
    }
    if let Some(op) = sha512_three_reg(raw) {
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
    if (raw & 0xFFFF_FC00) == 0xCEC0_8400 {
        return DecodeStep::Hit(Instr {
            op: Opcode::SimdSm4e,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: 0,
            imm: 0,
            sf: true,
            cond: 0,
            size: 16,
        });
    }
    if (raw & 0xFFE0_FC00) == 0xCE60_C000 {
        return DecodeStep::Hit(Instr {
            op: Opcode::SimdSm3Partw1,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: ((raw >> 16) & 0x1F) as u8,
            imm: 0,
            sf: true,
            cond: 0,
            size: 16,
        });
    }
    if (raw & 0xFFE0_8000) == 0xCE00_0000 {
        return DecodeStep::Hit(Instr {
            op: Opcode::SimdEor3,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: ((raw >> 16) & 0x1F) as u8,
            imm: 0,
            sf: true,
            cond: ((raw >> 10) & 0x1F) as u8,
            size: 16,
        });
    }
    if (raw & 0xFFE0_8000) == 0xCE20_0000 {
        return DecodeStep::Hit(Instr {
            op: Opcode::SimdBcax,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: ((raw >> 16) & 0x1F) as u8,
            imm: 0,
            sf: true,
            cond: ((raw >> 10) & 0x1F) as u8,
            size: 16,
        });
    }
    if (raw & 0xFFE0_FC00) == 0xCE60_8C00 {
        return DecodeStep::Hit(Instr {
            op: Opcode::SimdRax1,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: ((raw >> 16) & 0x1F) as u8,
            imm: 0,
            sf: true,
            cond: 0,
            size: 16,
        });
    }
    if (raw & 0xFFE0_0000) == 0xCE80_0000 {
        return DecodeStep::Hit(Instr {
            op: Opcode::SimdXar,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: ((raw >> 16) & 0x1F) as u8,
            imm: ((raw >> 10) & 0x3F) as u64,
            sf: true,
            cond: 0,
            size: 16,
        });
    }
    DecodeStep::Miss
}

fn sha512_three_reg(raw: u32) -> Option<Opcode> {
    match raw & 0xFFE0_FC00 {
        0xCE60_8000 => Some(Opcode::SimdSha512H),
        0xCE60_8400 => Some(Opcode::SimdSha512H2),
        0xCE60_8800 => Some(Opcode::SimdSha512Su1),
        _ => None,
    }
}
