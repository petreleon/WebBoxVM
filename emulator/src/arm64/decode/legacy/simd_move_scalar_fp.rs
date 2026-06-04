use super::*;

pub(super) fn decode(raw: u32) -> DecodeStep {
    if (raw & 0x7F3F_FC00) == 0x1E27_0000 {
        return DecodeStep::Hit(Instr {
            op: Opcode::SimdFmovGprToS,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: 0,
            imm: 0,
            sf: (raw >> 31) != 0,
            cond: 0,
            size: if (raw >> 31) != 0 { 8 } else { 4 },
        });
    }
    if (raw & 0xFFFF_FC00) == 0x9EAE_0000 {
        return DecodeStep::Hit(Instr {
            op: Opcode::SimdFmovLaneToGpr,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: 0,
            imm: 1,
            sf: true,
            cond: 0,
            size: 8,
        });
    }
    if (raw & 0xFFFF_FC00) == 0x9EAF_0000 {
        return DecodeStep::Hit(Instr {
            op: Opcode::SimdInsGprLane,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: 0,
            imm: 1,
            sf: true,
            cond: 0,
            size: 8,
        });
    }
    if let Some(instr) = decode_fp_scalar(raw) {
        return DecodeStep::Hit(instr);
    }
    if (raw & 0xBFE0_FC00) == 0x0E00_3C00 {
        let q = ((raw >> 30) & 1) != 0;
        let imm5 = ((raw >> 16) & 0x1F) as u8;
        if let Some((element_size, lane)) = decode_umov_element(imm5) {
            let data_size = if q { 8 } else { 4 };
            if (data_size == 8 && element_size == 8) || (data_size == 4 && element_size < 8) {
                return DecodeStep::Hit(Instr {
                    op: Opcode::SimdUmov,
                    rd: (raw & 0x1F) as u8,
                    rn: ((raw >> 5) & 0x1F) as u8,
                    rm: 0,
                    imm: lane as u64,
                    sf: q,
                    cond: element_size,
                    size: data_size,
                });
            }
        }
    }
    if (raw & 0xBFE0_FC00) == 0x0E00_2C00 {
        let q = ((raw >> 30) & 1) != 0;
        let imm5 = ((raw >> 16) & 0x1F) as u8;
        if let Some((element_size, lane)) = decode_umov_element(imm5) {
            let data_size = if q { 8 } else { 4 };
            if (element_size as usize) < data_size {
                return DecodeStep::Hit(Instr {
                    op: Opcode::SimdSmov,
                    rd: (raw & 0x1F) as u8,
                    rn: ((raw >> 5) & 0x1F) as u8,
                    rm: 0,
                    imm: lane as u64,
                    sf: q,
                    cond: element_size,
                    size: data_size as u8,
                });
            }
        }
    }
    if (raw & 0xFFE0_FC00) == 0x4E00_1C00 {
        return DecodeStep::Hit(Instr {
            op: Opcode::SimdInsGprLane,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: 0,
            imm: 1,
            sf: true,
            cond: 0,
            size: 8,
        });
    }
    if (raw & 0xBFBF_FC00) == 0x2EA0_F800 {
        let q = ((raw >> 30) & 1) != 0;
        let element_size = match (raw >> 22) & 0x3 {
            2 => 4,
            3 if q => 8,
            _ => return DecodeStep::Reject,
        };
        return DecodeStep::Hit(Instr {
            op: Opcode::SimdFpNeg,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: 0,
            imm: element_size,
            sf: true,
            cond: 0,
            size: if q { 16 } else { 8 },
        });
    }
    if (raw & 0xFF20_FC00) == 0x7E20_D400 {
        return DecodeStep::Hit(Instr {
            op: Opcode::SimdFpAbd,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: ((raw >> 16) & 0x1F) as u8,
            imm: if ((raw >> 22) & 1) != 0 { 8 } else { 4 },
            sf: true,
            cond: 0,
            size: if ((raw >> 22) & 1) != 0 { 8 } else { 4 },
        });
    }
    if let Some(instr) = decode_simd_fp_binary(raw, 0x0E20_D400, Opcode::SimdFpAddVec) {
        return DecodeStep::Hit(instr);
    }
    if let Some(instr) = decode_simd_fp_binary(raw, 0x0EA0_D400, Opcode::SimdFpSubVec) {
        return DecodeStep::Hit(instr);
    }
    if let Some(instr) = decode_simd_fp_binary(raw, 0x2E20_DC00, Opcode::SimdFpMulVec) {
        return DecodeStep::Hit(instr);
    }
    if let Some(instr) = decode_simd_fp_binary(raw, 0x0E20_CC00, Opcode::SimdFpFmlaVec) {
        return DecodeStep::Hit(instr);
    }
    if let Some(instr) = decode_simd_fp_binary(raw, 0x0EA0_CC00, Opcode::SimdFpFmlsVec) {
        return DecodeStep::Hit(instr);
    }
    DecodeStep::Miss
}
