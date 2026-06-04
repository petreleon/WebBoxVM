use super::*;

pub(super) fn decode(raw: u32) -> DecodeStep {
    if (raw & 0xBFF8_FC00) == 0x0F00_F400 {
        let imm8 = ((raw >> 5) & 0x1F) | (((raw >> 16) & 0x7) << 5);
        return DecodeStep::Hit(Instr {
            op: Opcode::SimdFmovImm,
            rd: (raw & 0x1F) as u8,
            rn: 0,
            rm: 0,
            imm: imm8 as u64,
            sf: true,
            cond: 4,
            size: if (raw >> 30) != 0 { 16 } else { 8 },
        });
    }
    if (raw & 0xFFF8_FC00) == 0x6F00_F400 {
        let imm8 = ((raw >> 5) & 0x1F) | (((raw >> 16) & 0x7) << 5);
        return DecodeStep::Hit(Instr {
            op: Opcode::SimdFmovImm,
            rd: (raw & 0x1F) as u8,
            rn: 0,
            rm: 0,
            imm: imm8 as u64,
            sf: true,
            cond: 8,
            size: 16,
        });
    }
    if (raw & 0xBFF8_9C00) == 0x0F00_0400 {
        let imm8 = ((raw >> 5) & 0x1F) | (((raw >> 16) & 0x7) << 5);
        let shift = (((raw >> 12) & 0xF) >> 1) & 0x3;
        return DecodeStep::Hit(Instr {
            op: Opcode::SimdMovi,
            rd: (raw & 0x1F) as u8,
            rn: 0,
            rm: 0,
            imm: (imm8 << (shift * 8)) as u64,
            sf: true,
            cond: 4,
            size: if (raw >> 30) != 0 { 16 } else { 8 },
        });
    }
    if (raw & 0xBFF8_DC00) == 0x0F00_8400 {
        let imm8 = ((raw >> 5) & 0x1F) | (((raw >> 16) & 0x7) << 5);
        let shift = (((raw >> 12) & 0x2) >> 1) * 8;
        return DecodeStep::Hit(Instr {
            op: Opcode::SimdMovi,
            rd: (raw & 0x1F) as u8,
            rn: 0,
            rm: 0,
            imm: (imm8 << shift) as u64,
            sf: true,
            cond: 2,
            size: if (raw >> 30) != 0 { 16 } else { 8 },
        });
    }
    if (raw & 0xFFFF_FC00) == 0x6F00_0400 {
        return DecodeStep::Hit(Instr {
            op: Opcode::SimdMovi,
            rd: (raw & 0x1F) as u8,
            rn: 0,
            rm: 0,
            imm: u64::MAX,
            sf: true,
            cond: 0,
            size: 16,
        });
    }
    if (raw & 0xBFF8_FC00) == 0x2F00_E400 {
        let imm8 = ((raw >> 5) & 0x1F) | (((raw >> 16) & 0x7) << 5);
        return DecodeStep::Hit(Instr {
            op: Opcode::SimdMovi,
            rd: (raw & 0x1F) as u8,
            rn: 0,
            rm: 0,
            imm: decode_movi_doubleword_imm(imm8),
            sf: true,
            cond: 8,
            size: if (raw >> 30) != 0 { 16 } else { 8 },
        });
    }
    if (raw & 0xBFF8_FC00) == 0x0F00_E400 {
        let imm8 = ((raw >> 5) & 0x1F) | (((raw >> 16) & 0x7) << 5);
        return DecodeStep::Hit(Instr {
            op: Opcode::SimdMovi,
            rd: (raw & 0x1F) as u8,
            rn: 0,
            rm: 0,
            imm: imm8 as u64,
            sf: true,
            cond: 0,
            size: if (raw >> 30) != 0 { 16 } else { 8 },
        });
    }
    if (raw & 0xBFF8_DC00) == 0x2F00_8400 {
        let imm8 = ((raw >> 5) & 0x1F) | (((raw >> 16) & 0x7) << 5);
        let shift = (((raw >> 12) & 0x2) >> 1) * 8;
        return DecodeStep::Hit(Instr {
            op: Opcode::SimdMvni,
            rd: (raw & 0x1F) as u8,
            rn: 0,
            rm: 0,
            imm: (imm8 << shift) as u64,
            sf: true,
            cond: 2,
            size: if (raw >> 30) != 0 { 16 } else { 8 },
        });
    }
    if (raw & 0xBFF8_9C00) == 0x2F00_0400 {
        let imm8 = ((raw >> 5) & 0x1F) | (((raw >> 16) & 0x7) << 5);
        let shift = (((raw >> 12) & 0xF) >> 1) & 0x3;
        return DecodeStep::Hit(Instr {
            op: Opcode::SimdMvni,
            rd: (raw & 0x1F) as u8,
            rn: 0,
            rm: 0,
            imm: (imm8 << (shift * 8)) as u64,
            sf: true,
            cond: 4,
            size: if (raw >> 30) != 0 { 16 } else { 8 },
        });
    }
    if (raw & 0xBFF8_EC00) == 0x2F00_C400 {
        let imm8 = ((raw >> 5) & 0x1F) | (((raw >> 16) & 0x7) << 5);
        let shift = if ((raw >> 12) & 1) == 0 { 8 } else { 16 };
        let ones = (1u64 << shift) - 1;
        return DecodeStep::Hit(Instr {
            op: Opcode::SimdMvni,
            rd: (raw & 0x1F) as u8,
            rn: 0,
            rm: 0,
            imm: ((imm8 as u64) << shift) | ones,
            sf: true,
            cond: 4,
            size: if (raw >> 30) != 0 { 16 } else { 8 },
        });
    }
    if (raw & 0xFFF8_FC00) == 0x4F00_E400 {
        let imm8 = ((raw >> 5) & 0x1F) | (((raw >> 16) & 0x7) << 5);
        return DecodeStep::Hit(Instr {
            op: Opcode::SimdMovi,
            rd: (raw & 0x1F) as u8,
            rn: 0,
            rm: 0,
            imm: imm8 as u64,
            sf: true,
            cond: 0,
            size: 16,
        });
    }
    DecodeStep::Miss
}
