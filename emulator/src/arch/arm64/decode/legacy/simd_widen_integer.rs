use super::*;

pub(super) fn decode(raw: u32) -> DecodeStep {
    if (raw & 0xBF20_FC00) == 0x0E20_1000 {
        let element_size = 1u64 << ((raw >> 22) & 0x3);
        if element_size < 8 {
            return DecodeStep::Hit(Instr {
                op: Opcode::SimdSaddw,
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
    if (raw & 0xBF20_FC00) == 0x2E20_1000 {
        let element_size = 1u64 << ((raw >> 22) & 0x3);
        if element_size < 8 {
            return DecodeStep::Hit(Instr {
                op: Opcode::SimdUaddw,
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
    if (raw & 0xBF20_FC00) == 0x0E20_3000 {
        let element_size = 1u64 << ((raw >> 22) & 0x3);
        if element_size < 8 {
            return DecodeStep::Hit(Instr {
                op: Opcode::SimdSsubw,
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
    if (raw & 0xBF20_FC00) == 0x2E20_3000 {
        let element_size = 1u64 << ((raw >> 22) & 0x3);
        if element_size < 8 {
            return DecodeStep::Hit(Instr {
                op: Opcode::SimdUsubw,
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
    if let Some(instr) = decode_simd_widen_mul_by_element(raw) {
        return DecodeStep::Hit(instr);
    }
    if let Some(instr) = decode_simd_widen_mul_vector(raw) {
        return DecodeStep::Hit(instr);
    }
    if (raw & 0xBF20_FC00) == 0x0E20_8400 {
        let element_size = 1u64 << ((raw >> 22) & 0x3);
        return DecodeStep::Hit(Instr {
            op: Opcode::SimdAddVec,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: ((raw >> 16) & 0x1F) as u8,
            imm: element_size,
            sf: true,
            cond: 0,
            size: if (raw >> 30) != 0 { 16 } else { 8 },
        });
    }
    if (raw & 0xBF20_FC00) == 0x2E20_8400 {
        let element_size = 1u64 << ((raw >> 22) & 0x3);
        return DecodeStep::Hit(Instr {
            op: Opcode::SimdSubVec,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: ((raw >> 16) & 0x1F) as u8,
            imm: element_size,
            sf: true,
            cond: 0,
            size: if (raw >> 30) != 0 { 16 } else { 8 },
        });
    }
    if (raw & 0xBF20_FC00) == 0x0E20_9C00 {
        let element_size = 1u64 << ((raw >> 22) & 0x3);
        if element_size < 8 {
            return DecodeStep::Hit(Instr {
                op: Opcode::SimdMulVec,
                rd: (raw & 0x1F) as u8,
                rn: ((raw >> 5) & 0x1F) as u8,
                rm: ((raw >> 16) & 0x1F) as u8,
                imm: element_size,
                sf: true,
                cond: 0,
                size: if (raw >> 30) != 0 { 16 } else { 8 },
            });
        }
    }
    if (raw & 0xBF20_FC00) == 0x0E20_9400 {
        let element_size = 1u64 << ((raw >> 22) & 0x3);
        if element_size < 8 {
            return DecodeStep::Hit(Instr {
                op: Opcode::SimdMlaVec,
                rd: (raw & 0x1F) as u8,
                rn: ((raw >> 5) & 0x1F) as u8,
                rm: ((raw >> 16) & 0x1F) as u8,
                imm: element_size,
                sf: true,
                cond: 0,
                size: if (raw >> 30) != 0 { 16 } else { 8 },
            });
        }
    }
    if (raw & 0xBF20_FC00) == 0x2E20_9400 {
        let element_size = 1u64 << ((raw >> 22) & 0x3);
        if element_size < 8 {
            return DecodeStep::Hit(Instr {
                op: Opcode::SimdMlsVec,
                rd: (raw & 0x1F) as u8,
                rn: ((raw >> 5) & 0x1F) as u8,
                rm: ((raw >> 16) & 0x1F) as u8,
                imm: element_size,
                sf: true,
                cond: 0,
                size: if (raw >> 30) != 0 { 16 } else { 8 },
            });
        }
    }
    if (raw & 0xBF20_FC00) == 0x0E20_8C00 {
        let element_size = 1u64 << ((raw >> 22) & 0x3);
        return DecodeStep::Hit(Instr {
            op: Opcode::SimdCmtst,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: ((raw >> 16) & 0x1F) as u8,
            imm: element_size,
            sf: true,
            cond: 0,
            size: if (raw >> 30) != 0 { 16 } else { 8 },
        });
    }
    DecodeStep::Miss
}
