use super::*;

pub(super) fn decode(raw: u32) -> DecodeStep {
    if (raw & 0xBFE0_FC00) == 0x0E00_0C00 {
        let q = ((raw >> 30) & 1) != 0;
        let imm5 = ((raw >> 16) & 0x1F) as u8;
        if let Some((element_size, _)) = decode_umov_element(imm5) {
            if element_size < 8 || q {
                return DecodeStep::Hit(Instr {
                    op: Opcode::SimdDupByte,
                    rd: (raw & 0x1F) as u8,
                    rn: ((raw >> 5) & 0x1F) as u8,
                    rm: 0,
                    imm: 0,
                    sf: true,
                    cond: element_size,
                    size: if q { 16 } else { 8 },
                });
            }
        }
    }
    if (raw & 0xFFBF_FC00) == 0x7EA1_B800 {
        let size = if ((raw >> 22) & 1) != 0 { 8 } else { 4 };
        return DecodeStep::Hit(Instr {
            op: Opcode::SimdFcvtzu,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: 0,
            imm: 0,
            sf: true,
            cond: 0,
            size,
        });
    }
    if let Some(instr) =
        decode_simd_int_fp_convert(raw, 0x5E21_D800, 0x0E21_D800, Opcode::SimdScvtf)
    {
        return DecodeStep::Hit(instr);
    }
    if let Some(instr) =
        decode_simd_int_fp_convert(raw, 0x7E21_D800, 0x2E21_D800, Opcode::SimdUcvtf)
    {
        return DecodeStep::Hit(instr);
    }
    if let Some(instr) =
        decode_simd_int_fp_convert(raw, 0x5EA1_B800, 0x0EA1_B800, Opcode::SimdFcvtzs)
    {
        return DecodeStep::Hit(instr);
    }
    if (raw & 0xBFE0_FC00) == 0x0E00_0400 {
        let q = ((raw >> 30) & 1) != 0;
        let imm5 = ((raw >> 16) & 0x1F) as u8;
        if let Some((element_size, lane)) = decode_umov_element(imm5) {
            if element_size < 8 || q {
                return DecodeStep::Hit(Instr {
                    op: Opcode::SimdDupElem,
                    rd: (raw & 0x1F) as u8,
                    rn: ((raw >> 5) & 0x1F) as u8,
                    rm: 0,
                    imm: lane as u64,
                    sf: true,
                    cond: element_size,
                    size: if q { 16 } else { 8 },
                });
            }
        }
    }
    if (raw & 0xFFE0_FC00) == 0x5E00_0400 {
        let imm5 = ((raw >> 16) & 0x1F) as u8;
        if let Some((element_size, lane)) = decode_umov_element(imm5) {
            return DecodeStep::Hit(Instr {
                op: Opcode::SimdDupElem,
                rd: (raw & 0x1F) as u8,
                rn: ((raw >> 5) & 0x1F) as u8,
                rm: 0,
                imm: lane as u64,
                sf: true,
                cond: element_size,
                size: element_size,
            });
        }
    }
    if (raw & 0xFFBF_FC00) == 0x1E20_4000 {
        let size = if ((raw >> 22) & 1) != 0 { 8 } else { 4 };
        return DecodeStep::Hit(Instr {
            op: Opcode::SimdFmovReg64,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: 0,
            imm: 0,
            sf: true,
            cond: 0,
            size,
        });
    }
    if (raw & 0xFFFF_FC00) == 0x9E67_0000 {
        return DecodeStep::Hit(Instr {
            op: Opcode::SimdFmovGprToD,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: 0,
            imm: 0,
            sf: true,
            cond: 0,
            size: 8,
        });
    }
    if (raw & 0xFFFF_FC00) == 0x9E66_0000 {
        return DecodeStep::Hit(Instr {
            op: Opcode::SimdFmovDToGpr,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: 0,
            imm: 0,
            sf: true,
            cond: 0,
            size: 8,
        });
    }
    if (raw & 0x7F3F_FC00) == 0x1E26_0000 {
        return DecodeStep::Hit(Instr {
            op: Opcode::SimdFmovSToGpr,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: 0,
            imm: 0,
            sf: (raw >> 31) != 0,
            cond: 0,
            size: if (raw >> 31) != 0 { 8 } else { 4 },
        });
    }
    DecodeStep::Miss
}
