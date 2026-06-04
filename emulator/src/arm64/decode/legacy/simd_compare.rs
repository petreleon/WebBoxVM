use super::*;

pub(super) fn decode(raw: u32) -> DecodeStep {
    if (raw & 0xBF3F_FC00) == 0x2E20_8800 {
        let q = ((raw >> 30) & 1) != 0;
        let element_size = 1u64 << ((raw >> 22) & 0x3);
        if element_size == 8 && !q {
            return DecodeStep::Reject;
        }
        return DecodeStep::Hit(Instr {
            op: Opcode::SimdCmgeZero,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: 0,
            imm: element_size,
            sf: true,
            cond: 0,
            size: if q { 16 } else { 8 },
        });
    }
    if (raw & 0xBF20_FC00) == 0x2E20_8C00 {
        let element_size = 1u64 << ((raw >> 22) & 0x3);
        return DecodeStep::Hit(Instr {
            op: Opcode::SimdCmeqReg,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: ((raw >> 16) & 0x1F) as u8,
            imm: element_size,
            sf: true,
            cond: 0,
            size: if (raw >> 30) != 0 { 16 } else { 8 },
        });
    }
    if (raw & 0xFFE0_FC00) == 0x7EE0_3400 {
        return DecodeStep::Hit(Instr {
            op: Opcode::SimdCmhiReg,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: ((raw >> 16) & 0x1F) as u8,
            imm: 8,
            sf: true,
            cond: 0,
            size: 8,
        });
    }
    if (raw & 0xBF20_FC00) == 0x2E20_3400 {
        let q = (raw >> 30) != 0;
        let element_size = 1u64 << ((raw >> 22) & 0x3);
        if element_size == 8 && !q {
            return DecodeStep::Reject;
        }
        return DecodeStep::Hit(Instr {
            op: Opcode::SimdCmhiReg,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: ((raw >> 16) & 0x1F) as u8,
            imm: element_size,
            sf: true,
            cond: 0,
            size: if q { 16 } else { 8 },
        });
    }
    if (raw & 0xBF20_FC00) == 0x2E20_3C00 {
        let q = (raw >> 30) != 0;
        let element_size = 1u64 << ((raw >> 22) & 0x3);
        if element_size == 8 && !q {
            return DecodeStep::Reject;
        }
        return DecodeStep::Hit(Instr {
            op: Opcode::SimdCmhsReg,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: ((raw >> 16) & 0x1F) as u8,
            imm: element_size,
            sf: true,
            cond: 0,
            size: if q { 16 } else { 8 },
        });
    }
    if (raw & 0xFF20_FC00) == 0x7E20_2C00 {
        let element_size = 1u64 << ((raw >> 22) & 0x3);
        return DecodeStep::Hit(Instr {
            op: Opcode::SimdUqsub,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: ((raw >> 16) & 0x1F) as u8,
            imm: element_size,
            sf: true,
            cond: 0,
            size: element_size as u8,
        });
    }
    if (raw & 0xBF20_FC00) == 0x2E20_3800 {
        let element_size = 1u64 << ((raw >> 22) & 0x3);
        if element_size < 8 {
            return DecodeStep::Hit(Instr {
                op: Opcode::SimdShll,
                rd: (raw & 0x1F) as u8,
                rn: ((raw >> 5) & 0x1F) as u8,
                rm: 0,
                imm: element_size * 8,
                sf: (raw >> 30) != 0,
                cond: element_size as u8,
                size: 16,
            });
        }
    }
    if (raw & 0xBF20_FC00) == 0x0E20_0000 {
        let element_size = 1u64 << ((raw >> 22) & 0x3);
        if element_size < 8 {
            return DecodeStep::Hit(Instr {
                op: Opcode::SimdSaddl,
                rd: (raw & 0x1F) as u8,
                rn: ((raw >> 5) & 0x1F) as u8,
                rm: ((raw >> 16) & 0x1F) as u8,
                imm: 0,
                sf: (raw >> 30) != 0,
                cond: element_size as u8,
                size: 16,
            });
        }
    }
    if (raw & 0xBF20_FC00) == 0x2E20_2000 {
        let element_size = 1u64 << ((raw >> 22) & 0x3);
        if element_size < 8 {
            return DecodeStep::Hit(Instr {
                op: Opcode::SimdUsubl,
                rd: (raw & 0x1F) as u8,
                rn: ((raw >> 5) & 0x1F) as u8,
                rm: ((raw >> 16) & 0x1F) as u8,
                imm: 0,
                sf: (raw >> 30) != 0,
                cond: element_size as u8,
                size: 16,
            });
        }
    }
    DecodeStep::Miss
}
