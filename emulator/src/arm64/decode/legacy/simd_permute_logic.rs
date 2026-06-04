use super::*;

pub(super) fn decode(raw: u32) -> DecodeStep {
    if (raw & 0xBFFF_FC00) == 0x2E20_5800 {
        return DecodeStep::Hit(Instr {
            op: Opcode::SimdNot,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: 0,
            imm: 0,
            sf: true,
            cond: 0,
            size: if (raw >> 30) != 0 { 16 } else { 8 },
        });
    }
    if (raw & 0xBFFF_FC00) == 0x2E60_5800 {
        return DecodeStep::Hit(Instr {
            op: Opcode::SimdRbit,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: 0,
            imm: 0,
            sf: true,
            cond: 0,
            size: if (raw >> 30) != 0 { 16 } else { 8 },
        });
    }
    if (raw & 0xFFE0_8400) == 0x6E00_0400 {
        let imm5 = ((raw >> 16) & 0x1F) as u8;
        if let Some((element_size, dest_lane)) = decode_umov_element(imm5) {
            let source_lane = (((raw >> 11) & 0xF) as u8) >> element_size.trailing_zeros();
            return DecodeStep::Hit(Instr {
                op: Opcode::SimdInsElem,
                rd: (raw & 0x1F) as u8,
                rn: ((raw >> 5) & 0x1F) as u8,
                rm: 0,
                imm: ((dest_lane as u64) << 8) | source_lane as u64,
                sf: true,
                cond: element_size,
                size: 16,
            });
        }
    }
    if (raw & 0xBFE0_FC00) == 0x2E60_1C00 {
        return DecodeStep::Hit(Instr {
            op: Opcode::SimdBsl,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: ((raw >> 16) & 0x1F) as u8,
            imm: 0,
            sf: (raw >> 30) != 0,
            cond: 0,
            size: if (raw >> 30) != 0 { 16 } else { 8 },
        });
    }
    if (raw & 0xBFE0_FC00) == 0x2EA0_1C00 {
        return DecodeStep::Hit(Instr {
            op: Opcode::SimdBit,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: ((raw >> 16) & 0x1F) as u8,
            imm: 0,
            sf: (raw >> 30) != 0,
            cond: 0,
            size: if (raw >> 30) != 0 { 16 } else { 8 },
        });
    }
    if (raw & 0xBFE0_FC00) == 0x2EE0_1C00 {
        return DecodeStep::Hit(Instr {
            op: Opcode::SimdBif,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: ((raw >> 16) & 0x1F) as u8,
            imm: 0,
            sf: (raw >> 30) != 0,
            cond: 0,
            size: if (raw >> 30) != 0 { 16 } else { 8 },
        });
    }
    if (raw & 0xBFE0_FC00) == 0x0E20_1C00 {
        return DecodeStep::Hit(Instr {
            op: Opcode::SimdAnd,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: ((raw >> 16) & 0x1F) as u8,
            imm: 0,
            sf: (raw >> 30) != 0,
            cond: 0,
            size: if (raw >> 30) != 0 { 16 } else { 8 },
        });
    }
    if (raw & 0xBFE0_FC00) == 0x0E60_1C00 {
        return DecodeStep::Hit(Instr {
            op: Opcode::SimdBic,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: ((raw >> 16) & 0x1F) as u8,
            imm: 0,
            sf: (raw >> 30) != 0,
            cond: 0,
            size: if (raw >> 30) != 0 { 16 } else { 8 },
        });
    }
    if (raw & 0xBFE0_FC00) == 0x0EA0_1C00 {
        return DecodeStep::Hit(Instr {
            op: Opcode::SimdOrr,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: ((raw >> 16) & 0x1F) as u8,
            imm: 0,
            sf: (raw >> 30) != 0,
            cond: 0,
            size: if (raw >> 30) != 0 { 16 } else { 8 },
        });
    }
    if (raw & 0xBFE0_FC00) == 0x0EE0_1C00 {
        return DecodeStep::Hit(Instr {
            op: Opcode::SimdOrn,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: ((raw >> 16) & 0x1F) as u8,
            imm: 0,
            sf: (raw >> 30) != 0,
            cond: 0,
            size: if (raw >> 30) != 0 { 16 } else { 8 },
        });
    }
    if (raw & 0xFFE0_FC00) == 0x6E20_1C00 || (raw & 0xFFE0_FC00) == 0x2E20_1C00 {
        return DecodeStep::Hit(Instr {
            op: Opcode::SimdEor,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: ((raw >> 16) & 0x1F) as u8,
            imm: 0,
            sf: (raw >> 30) != 0,
            cond: 0,
            size: if (raw >> 30) != 0 { 16 } else { 8 },
        });
    }
    if let Some(instr) = decode_simd_bic_imm(raw) {
        return DecodeStep::Hit(instr);
    }
    DecodeStep::Miss
}
