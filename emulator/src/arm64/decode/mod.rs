//! AArch64 instruction decoder (pattern-based).
//!
//! Primary decoder: disarm64 (spec-driven, 3000+ instructions, 2x faster).
//! Fallback: our hand-rolled decoder for instructions disarm64 doesn't handle.

mod branch;
mod data_proc;
mod disarm64_shim;
mod ldst;
mod system;

use super::opcodes::{Instr, Opcode};

/// Decode a raw 32-bit word into an instruction.
pub fn decode(raw: u32) -> Option<Instr> {
    disarm64_shim::decode(raw)
}

/// Legacy hand-rolled decoder (fallback within the shim).
pub(crate) fn decode_legacy(raw: u32) -> Option<Instr> {
    if raw == 0xD503_201F {
        return system::decode_nop();
    }
    if let Some(instr) = decode_simd_ld1_multi(raw) {
        return Some(instr);
    }
    if let Some(instr) = decode_simd_ldst1_lane(raw) {
        return Some(instr);
    }
    let sve_ldst_base = raw & 0xFFC0_E000;
    if matches!(
        sve_ldst_base,
        0x8580_0000 | 0x8580_4000 | 0xE580_0000 | 0xE580_4000
    ) {
        let imm9 = ((((raw >> 16) & 0x3F) << 3) | ((raw >> 10) & 0x7)) as u16;
        let signed_imm = ((imm9 as i16) << 7) >> 7;
        return Some(Instr {
            op: if (raw & 0x4000_0000) == 0 {
                Opcode::SveLdr
            } else {
                Opcode::SveStr
            },
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: 0,
            imm: signed_imm as i64 as u64,
            sf: true,
            cond: if (raw & 0x4000) != 0 { 1 } else { 0 },
            size: 0,
        });
    }
    if (raw & 0xFFFF_FC00) == 0x0420_BC00 {
        return Some(Instr {
            op: Opcode::SveMovprfx,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: 0,
            imm: 0,
            sf: false,
            cond: 0xFF,
            size: 0,
        });
    }
    if (raw & 0xFF3E_E000) == 0x0410_2000 {
        return Some(Instr {
            op: Opcode::SveMovprfx,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: 0,
            imm: 0,
            sf: (raw & 0x0001_0000) != 0,
            cond: ((raw >> 10) & 0x7) as u8,
            size: 1u8 << (((raw >> 22) & 0x3) as u8),
        });
    }
    if (raw & 0xFF3F_FC00) == 0x0520_3800 {
        return Some(Instr {
            op: Opcode::SveDupGpr,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: 0,
            imm: 0,
            sf: true,
            cond: 0,
            size: 1u8 << (((raw >> 22) & 0x3) as u8),
        });
    }
    let sve_addsub_base = raw & 0xFF20_FC00;
    if sve_addsub_base == 0x0420_0000 || sve_addsub_base == 0x0420_0400 {
        return Some(Instr {
            op: if sve_addsub_base == 0x0420_0000 {
                Opcode::SveAddVec
            } else {
                Opcode::SveSubVec
            },
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: ((raw >> 16) & 0x1F) as u8,
            imm: 0,
            sf: false,
            cond: 0,
            size: 1u8 << (((raw >> 22) & 0x3) as u8),
        });
    }
    let sve_logical_base = raw & 0xFFE0_FC00;
    if sve_logical_base == 0x0460_3000 || sve_logical_base == 0x04A0_3000 {
        return Some(Instr {
            op: if sve_logical_base == 0x0460_3000 {
                Opcode::SveOrrVec
            } else {
                Opcode::SveEorVec
            },
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: ((raw >> 16) & 0x1F) as u8,
            imm: 0,
            sf: false,
            cond: 0,
            size: 8,
        });
    }
    if (raw & 0xFF20_C000) == 0x0520_C000 {
        return Some(Instr {
            op: Opcode::SveSel,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: ((raw >> 16) & 0x1F) as u8,
            imm: 0,
            sf: false,
            cond: ((raw >> 10) & 0xF) as u8,
            size: 1u8 << (((raw >> 22) & 0x3) as u8),
        });
    }
    if (raw & 0xFF3F_FC10) == 0x2518_E000 {
        let size_bits = ((raw >> 22) & 0x3) as u8;
        return Some(Instr {
            op: Opcode::SvePtrue,
            rd: (raw & 0xF) as u8,
            rn: 0,
            rm: 0,
            imm: 0,
            sf: false,
            cond: ((raw >> 5) & 0x1F) as u8,
            size: 1u8 << size_bits,
        });
    }
    if (raw & 0xFFFF_C21F) == 0x2550_C000 {
        return Some(Instr {
            op: Opcode::SvePtest,
            rd: ((raw >> 10) & 0xF) as u8,
            rn: ((raw >> 5) & 0xF) as u8,
            rm: 0,
            imm: 0,
            sf: true,
            cond: 0,
            size: 1,
        });
    }
    let pred_logical_base = raw & 0xFFF0_C210;
    if matches!(
        pred_logical_base,
        0x2500_4000 | 0x2540_4000 | 0x2580_4000 | 0x25C0_4000
    ) {
        return Some(Instr {
            op: if (raw & 0x0080_0000) == 0 {
                Opcode::SvePredAnd
            } else {
                Opcode::SvePredOrr
            },
            rd: (raw & 0xF) as u8,
            rn: ((raw >> 5) & 0xF) as u8,
            rm: ((raw >> 16) & 0xF) as u8,
            imm: 0,
            sf: (raw & 0x0040_0000) != 0,
            cond: ((raw >> 10) & 0xF) as u8,
            size: 1,
        });
    }
    if (raw & 0xFF20_E000) == 0x0420_E000 {
        let size_bits = ((raw >> 22) & 0x3) as u8;
        return Some(Instr {
            op: Opcode::SveCnt,
            rd: (raw & 0x1F) as u8,
            rn: 0,
            rm: 0,
            imm: (((raw >> 16) & 0xF) + 1) as u64,
            sf: true,
            cond: ((raw >> 5) & 0x1F) as u8,
            size: 1u8 << size_bits,
        });
    }
    if (raw & 0xFFE0_F800) == 0x0420_5000 || (raw & 0xFFE0_F800) == 0x0420_5800 {
        let imm6 = ((raw >> 5) & 0x3F) as u8;
        let signed_imm = ((imm6 as i8) << 2) >> 2;
        return Some(Instr {
            op: if (raw & 0x800) == 0 {
                Opcode::SveAddvl
            } else {
                Opcode::SveAddsvl
            },
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 16) & 0x1F) as u8,
            rm: 0,
            imm: signed_imm as i64 as u64,
            sf: true,
            cond: 0,
            size: 0,
        });
    }
    let ld1r_no_offset = (raw & 0xBFFF_F000) == 0x0D40_C000;
    let ld1r_post_index = (raw & 0xBFE0_F000) == 0x0DC0_C000;
    if ld1r_no_offset || ld1r_post_index {
        let element_size = 1u8 << (((raw >> 10) & 0x3) as u8);
        let rm_field = ((raw >> 16) & 0x1F) as u8;
        let (rm, imm) = if ld1r_post_index {
            if rm_field == 31 {
                (0xFE, element_size as u64)
            } else {
                (rm_field, 0)
            }
        } else {
            (0xFF, 0)
        };
        return Some(Instr {
            op: Opcode::SimdLd1r,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm,
            imm,
            sf: true,
            cond: element_size,
            size: if (raw >> 30) != 0 { 16 } else { 8 },
        });
    }
    if (raw & 0xBFE0_FC00) == 0x0E00_0C00 {
        let q = ((raw >> 30) & 1) != 0;
        let imm5 = ((raw >> 16) & 0x1F) as u8;
        if let Some((element_size, _)) = decode_umov_element(imm5) {
            if element_size < 8 || q {
                return Some(Instr {
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
        return Some(Instr {
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
        return Some(instr);
    }
    if let Some(instr) =
        decode_simd_int_fp_convert(raw, 0x5EA1_B800, 0x0EA1_B800, Opcode::SimdFcvtzs)
    {
        return Some(instr);
    }
    if (raw & 0xBFE0_FC00) == 0x0E00_0400 {
        let q = ((raw >> 30) & 1) != 0;
        let imm5 = ((raw >> 16) & 0x1F) as u8;
        if let Some((element_size, lane)) = decode_umov_element(imm5) {
            if element_size < 8 || q {
                return Some(Instr {
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
            return Some(Instr {
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
        return Some(Instr {
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
        return Some(Instr {
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
        return Some(Instr {
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
        return Some(Instr {
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
    if (raw & 0x7F3F_FC00) == 0x1E27_0000 {
        return Some(Instr {
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
        return Some(Instr {
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
        return Some(Instr {
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
        return Some(instr);
    }
    if (raw & 0xBFE0_FC00) == 0x0E00_3C00 {
        let q = ((raw >> 30) & 1) != 0;
        let imm5 = ((raw >> 16) & 0x1F) as u8;
        if let Some((element_size, lane)) = decode_umov_element(imm5) {
            let data_size = if q { 8 } else { 4 };
            if (data_size == 8 && element_size == 8) || (data_size == 4 && element_size < 8) {
                return Some(Instr {
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
                return Some(Instr {
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
        return Some(Instr {
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
            _ => return None,
        };
        return Some(Instr {
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
        return Some(Instr {
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
        return Some(instr);
    }
    if let Some(instr) = decode_simd_fp_binary(raw, 0x0EA0_D400, Opcode::SimdFpSubVec) {
        return Some(instr);
    }
    if let Some(instr) = decode_simd_fp_binary(raw, 0x2E20_DC00, Opcode::SimdFpMulVec) {
        return Some(instr);
    }
    if let Some(instr) = decode_simd_fp_binary(raw, 0x2E20_FC00, Opcode::SimdFpDivVec) {
        return Some(instr);
    }
    if let Some(instr) = decode_simd_fp_binary(raw, 0x2EA0_D400, Opcode::SimdFpAbd) {
        return Some(instr);
    }
    if (raw & 0xFF3F_FC00) == 0x7E20_B800 {
        return Some(Instr {
            op: Opcode::SimdNeg,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: 0,
            imm: 8,
            sf: true,
            cond: 0,
            size: 8,
        });
    }
    if (raw & 0xFFFF_FC00) == 0x5EE0_B800 {
        return Some(Instr {
            op: Opcode::SimdAbs,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: 0,
            imm: 8,
            sf: true,
            cond: 0,
            size: 8,
        });
    }
    if (raw & 0xFF3F_FC00) == 0x5E20_B800 {
        return None;
    }
    if (raw & 0xBF3F_FC00) == 0x0E20_B800 {
        let q = ((raw >> 30) & 1) != 0;
        let element_size = 1u64 << ((raw >> 22) & 0x3);
        if element_size == 8 && !q {
            return None;
        }
        return Some(Instr {
            op: Opcode::SimdAbs,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: 0,
            imm: element_size,
            sf: true,
            cond: 0,
            size: if q { 16 } else { 8 },
        });
    }
    if (raw & 0xBF3F_FC00) == 0x2E20_B800 {
        let q = ((raw >> 30) & 1) != 0;
        let element_size = 1u64 << ((raw >> 22) & 0x3);
        if element_size == 8 && !q {
            return None;
        }
        return Some(Instr {
            op: Opcode::SimdNeg,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: 0,
            imm: element_size,
            sf: true,
            cond: 0,
            size: if q { 16 } else { 8 },
        });
    }
    if (raw & 0xBF3F_FC00) == 0x0E20_9800 {
        let element_size = 1u64 << ((raw >> 22) & 0x3);
        let q = ((raw >> 30) & 1) != 0;
        if element_size == 8 && !q {
            return None;
        }
        return Some(Instr {
            op: Opcode::SimdCmeqZero,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: 0,
            imm: element_size,
            sf: true,
            cond: 0,
            size: if q { 16 } else { 8 },
        });
    }
    if (raw & 0xFF20_FC00) == 0x5E20_9800 {
        if ((raw >> 22) & 0x3) != 0x3 {
            return None;
        }
        return Some(Instr {
            op: Opcode::SimdCmeqZero,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: 0,
            imm: 8,
            sf: true,
            cond: 0,
            size: 8,
        });
    }
    if (raw & 0xFF3F_FC00) == 0x7E20_8800 {
        let element_size = 1u64 << ((raw >> 22) & 0x3);
        if element_size != 8 {
            return None;
        }
        return Some(Instr {
            op: Opcode::SimdCmgeZero,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: 0,
            imm: element_size,
            sf: true,
            cond: 0,
            size: 8,
        });
    }
    if (raw & 0xBF3F_FC00) == 0x2E20_8800 {
        let q = ((raw >> 30) & 1) != 0;
        let element_size = 1u64 << ((raw >> 22) & 0x3);
        if element_size == 8 && !q {
            return None;
        }
        return Some(Instr {
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
        return Some(Instr {
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
        return Some(Instr {
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
            return None;
        }
        return Some(Instr {
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
    if (raw & 0xFFE0_FC00) == 0x6E20_3C00 {
        return Some(Instr {
            op: Opcode::SimdCmhsReg,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: ((raw >> 16) & 0x1F) as u8,
            imm: 0,
            sf: true,
            cond: 0,
            size: 16,
        });
    }
    if (raw & 0xFF20_FC00) == 0x7E20_2C00 {
        let element_size = 1u64 << ((raw >> 22) & 0x3);
        return Some(Instr {
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
            return Some(Instr {
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
            return Some(Instr {
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
            return Some(Instr {
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
    if (raw & 0xBF20_FC00) == 0x0E20_3000 {
        let element_size = 1u64 << ((raw >> 22) & 0x3);
        if element_size < 8 {
            return Some(Instr {
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
    if let Some(instr) = decode_simd_umlal_by_element(raw) {
        return Some(instr);
    }
    if (raw & 0xBF20_FC00) == 0x0E20_8400 {
        let element_size = 1u64 << ((raw >> 22) & 0x3);
        return Some(Instr {
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
        return Some(Instr {
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
            return Some(Instr {
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
            return Some(Instr {
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
    if (raw & 0xBF20_FC00) == 0x0E20_8C00 {
        let element_size = 1u64 << ((raw >> 22) & 0x3);
        return Some(Instr {
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
    if (raw & 0xFFE0_FC00) == 0x7EE0_4400 {
        return Some(Instr {
            op: Opcode::SimdUshl,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: ((raw >> 16) & 0x1F) as u8,
            imm: 8,
            sf: true,
            cond: 0,
            size: 8,
        });
    }
    if (raw & 0xBF20_FC00) == 0x2E20_4400 {
        let q = (raw >> 30) != 0;
        let element_size = 1u64 << ((raw >> 22) & 0x3);
        if element_size == 8 && !q {
            return None;
        }
        return Some(Instr {
            op: Opcode::SimdUshl,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: ((raw >> 16) & 0x1F) as u8,
            imm: element_size,
            sf: true,
            cond: 0,
            size: if q { 16 } else { 8 },
        });
    }
    if (raw & 0xBF20_FC00) == 0x0E00_1800 {
        let element_size = 1u64 << ((raw >> 22) & 0x3);
        let vector_size = if (raw >> 30) != 0 { 16 } else { 8 };
        if element_size >= vector_size {
            return None;
        }
        return Some(Instr {
            op: Opcode::SimdUzp1,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: ((raw >> 16) & 0x1F) as u8,
            imm: element_size,
            sf: true,
            cond: 0,
            size: vector_size as u8,
        });
    }
    if (raw & 0xBF20_FC00) == 0x0E00_2800 {
        let element_size = 1u64 << ((raw >> 22) & 0x3);
        let vector_size = if (raw >> 30) != 0 { 16 } else { 8 };
        if element_size >= vector_size {
            return None;
        }
        return Some(Instr {
            op: Opcode::SimdTrn1,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: ((raw >> 16) & 0x1F) as u8,
            imm: element_size,
            sf: true,
            cond: 0,
            size: vector_size as u8,
        });
    }
    if (raw & 0xBF20_FC00) == 0x0E00_3800 {
        let element_size = 1u64 << ((raw >> 22) & 0x3);
        let vector_size = if (raw >> 30) != 0 { 16 } else { 8 };
        if element_size >= vector_size {
            return None;
        }
        return Some(Instr {
            op: Opcode::SimdZip1,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: ((raw >> 16) & 0x1F) as u8,
            imm: element_size,
            sf: true,
            cond: 0,
            size: vector_size as u8,
        });
    }
    if (raw & 0xBF20_FC00) == 0x0E00_7800 {
        let element_size = 1u64 << ((raw >> 22) & 0x3);
        let vector_size = if (raw >> 30) != 0 { 16 } else { 8 };
        if element_size >= vector_size {
            return None;
        }
        return Some(Instr {
            op: Opcode::SimdZip2,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: ((raw >> 16) & 0x1F) as u8,
            imm: element_size,
            sf: true,
            cond: 0,
            size: vector_size as u8,
        });
    }
    if (raw & 0xBFE0_9C00) == 0x0E00_0000 {
        return Some(Instr {
            op: Opcode::SimdTbl,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: ((raw >> 16) & 0x1F) as u8,
            imm: 0,
            sf: true,
            cond: (((raw >> 13) & 0x3) + 1) as u8,
            size: if (raw >> 30) != 0 { 16 } else { 8 },
        });
    }
    if (raw & 0xFFFF_FC00) == 0x4E28_4800 {
        return Some(Instr {
            op: Opcode::SimdAese,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: 0,
            imm: 0,
            sf: true,
            cond: 0,
            size: 16,
        });
    }
    if (raw & 0xFFFF_FC00) == 0x4E28_5800 {
        return Some(Instr {
            op: Opcode::SimdAesd,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: 0,
            imm: 0,
            sf: true,
            cond: 0,
            size: 16,
        });
    }
    if (raw & 0xFFFF_FC00) == 0x4E28_6800 {
        return Some(Instr {
            op: Opcode::SimdAesmc,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: 0,
            imm: 0,
            sf: true,
            cond: 0,
            size: 16,
        });
    }
    if (raw & 0xFFFF_FC00) == 0x4E28_7800 {
        return Some(Instr {
            op: Opcode::SimdAesimc,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: 0,
            imm: 0,
            sf: true,
            cond: 0,
            size: 16,
        });
    }
    if (raw & 0xBF20_FC00) == 0x0E20_E000 {
        let size_bits = (raw >> 22) & 0x3;
        if matches!(size_bits, 1 | 2) {
            return None;
        }
        return Some(Instr {
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
        return Some(Instr {
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
        return Some(Instr {
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
        return Some(Instr {
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
    if (raw & 0xFFFF_FC00) == 0xCEC0_8400 {
        return Some(Instr {
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
        return Some(Instr {
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
        return Some(Instr {
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
        return Some(Instr {
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
        return Some(Instr {
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
        return Some(Instr {
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
    if (raw & 0xFF20_FC00) == 0x5E20_8400 {
        if ((raw >> 22) & 0x3) != 0x3 {
            return None;
        }
        let element_size = 1u64 << ((raw >> 22) & 0x3);
        return Some(Instr {
            op: Opcode::SimdAddVec,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: ((raw >> 16) & 0x1F) as u8,
            imm: element_size,
            sf: true,
            cond: 0,
            size: element_size as u8,
        });
    }
    if let Some(instr) = decode_simd_shrn(raw) {
        return Some(instr);
    }
    if let Some(instr) = decode_simd_shl(raw) {
        return Some(instr);
    }
    if let Some(instr) = decode_simd_sli(raw) {
        return Some(instr);
    }
    if let Some(instr) = decode_simd_sri(raw) {
        return Some(instr);
    }
    if let Some(instr) = decode_simd_sshr(raw) {
        return Some(instr);
    }
    if let Some(instr) = decode_simd_ushr(raw) {
        return Some(instr);
    }
    if let Some(instr) = decode_simd_sshll(raw) {
        return Some(instr);
    }
    if let Some(instr) = decode_simd_ushll(raw) {
        return Some(instr);
    }
    if (raw & 0xFF3F_FC00) == 0x0E21_2800 {
        let dest_element_size = 1u64 << ((raw >> 22) & 0x3);
        return Some(Instr {
            op: Opcode::SimdXtn,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: 0,
            imm: dest_element_size,
            sf: true,
            cond: 0,
            size: 8,
        });
    }
    if (raw & 0xFFFF_FC00) == 0x5EF1_B800 {
        return Some(Instr {
            op: Opcode::SimdAddp,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: 0xFF,
            imm: 8,
            sf: true,
            cond: 0,
            size: 8,
        });
    }
    if (raw & 0xFFFF_FC00) == 0x0E21_4000 {
        return Some(Instr {
            op: Opcode::SimdAddhn,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: ((raw >> 16) & 0x1F) as u8,
            imm: 0,
            sf: false,
            cond: 0,
            size: 8,
        });
    }
    if (raw & 0xBF20_FC00) == 0x0E20_BC00 {
        let q = ((raw >> 30) & 1) != 0;
        let element_size = 1u64 << ((raw >> 22) & 0x3);
        if element_size == 8 && !q {
            return None;
        }
        return Some(Instr {
            op: Opcode::SimdAddp,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: ((raw >> 16) & 0x1F) as u8,
            imm: element_size,
            sf: true,
            cond: 0,
            size: if q { 16 } else { 8 },
        });
    }
    if (raw & 0xBF3F_FC00) == 0x0E31_B800 {
        let element_size = 1u64 << ((raw >> 22) & 0x3);
        return Some(Instr {
            op: Opcode::SimdAddv,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: 0,
            imm: element_size,
            sf: true,
            cond: 0,
            size: if (raw >> 30) != 0 { 16 } else { 8 },
        });
    }
    if (raw & 0xBF3F_FC00) == 0x2E30_A800 {
        let q = ((raw >> 30) & 1) != 0;
        let element_size = 1u64 << ((raw >> 22) & 0x3);
        if element_size == 8 || (element_size == 4 && !q) {
            return None;
        }
        return Some(Instr {
            op: Opcode::SimdUmaxv,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: 0,
            imm: element_size,
            sf: true,
            cond: 0,
            size: if q { 16 } else { 8 },
        });
    }
    if (raw & 0xBFE0_8400) == 0x2E00_0000 {
        return Some(Instr {
            op: Opcode::SimdExt,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: ((raw >> 16) & 0x1F) as u8,
            imm: ((raw >> 11) & 0xF) as u64,
            sf: true,
            cond: 0,
            size: if (raw >> 30) != 0 { 16 } else { 8 },
        });
    }
    if (raw & 0xFF20_FC00) == 0x6E20_A400 {
        let element_size = 1u64 << ((raw >> 22) & 0x3);
        return Some(Instr {
            op: Opcode::SimdUmaxp,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: ((raw >> 16) & 0x1F) as u8,
            imm: element_size,
            sf: true,
            cond: 0,
            size: 16,
        });
    }
    if (raw & 0xBF20_FC00) == 0x0E20_6400 {
        let element_size = 1u64 << ((raw >> 22) & 0x3);
        if element_size == 8 {
            return None;
        }
        return Some(Instr {
            op: Opcode::SimdSmaxVec,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: ((raw >> 16) & 0x1F) as u8,
            imm: element_size,
            sf: true,
            cond: 0,
            size: if (raw >> 30) != 0 { 16 } else { 8 },
        });
    }
    if (raw & 0xBF20_FC00) == 0x2E20_6400 {
        let element_size = 1u64 << ((raw >> 22) & 0x3);
        if element_size == 8 {
            return None;
        }
        return Some(Instr {
            op: Opcode::SimdUmaxVec,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: ((raw >> 16) & 0x1F) as u8,
            imm: element_size,
            sf: true,
            cond: 0,
            size: if (raw >> 30) != 0 { 16 } else { 8 },
        });
    }
    if (raw & 0xBF20_FC00) == 0x2E20_6C00 {
        let element_size = 1u64 << ((raw >> 22) & 0x3);
        if element_size == 8 {
            return None;
        }
        return Some(Instr {
            op: Opcode::SimdUminVec,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: ((raw >> 16) & 0x1F) as u8,
            imm: element_size,
            sf: true,
            cond: 0,
            size: if (raw >> 30) != 0 { 16 } else { 8 },
        });
    }
    if (raw & 0xBF20_FC00) == 0x2E20_AC00 {
        let element_size = 1u64 << ((raw >> 22) & 0x3);
        return Some(Instr {
            op: Opcode::SimdUminp,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: ((raw >> 16) & 0x1F) as u8,
            imm: element_size,
            sf: true,
            cond: 0,
            size: if (raw >> 30) != 0 { 16 } else { 8 },
        });
    }
    if (raw & 0xBF3F_FC00) == 0x0E20_5800 {
        return Some(Instr {
            op: Opcode::SimdCnt,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: 0,
            imm: 0,
            sf: true,
            cond: 0,
            size: if (raw >> 30) != 0 { 16 } else { 8 },
        });
    }
    if (raw & 0xBF3F_FC00) == 0x2E20_0800 {
        let element_size = 1u64 << ((raw >> 22) & 0x3);
        return Some(Instr {
            op: Opcode::SimdRev32,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: 0,
            imm: element_size,
            sf: true,
            cond: 0,
            size: if (raw >> 30) != 0 { 16 } else { 8 },
        });
    }
    if (raw & 0xBF3F_FC00) == 0x0E20_0800 {
        let element_size = 1u64 << ((raw >> 22) & 0x3);
        if element_size < 8 {
            return Some(Instr {
                op: Opcode::SimdRev64,
                rd: (raw & 0x1F) as u8,
                rn: ((raw >> 5) & 0x1F) as u8,
                rm: 0,
                imm: element_size,
                sf: true,
                cond: 0,
                size: if (raw >> 30) != 0 { 16 } else { 8 },
            });
        }
    }
    if (raw & 0xBFFF_FC00) == 0x2E20_5800 {
        return Some(Instr {
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
    if (raw & 0xFFE0_8400) == 0x6E00_0400 {
        let imm5 = ((raw >> 16) & 0x1F) as u8;
        if let Some((element_size, dest_lane)) = decode_umov_element(imm5) {
            let source_lane = (((raw >> 11) & 0xF) as u8) >> element_size.trailing_zeros();
            return Some(Instr {
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
        return Some(Instr {
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
        return Some(Instr {
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
        return Some(Instr {
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
        return Some(Instr {
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
        return Some(Instr {
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
        return Some(Instr {
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
        return Some(Instr {
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
        return Some(Instr {
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
        return Some(instr);
    }
    if (raw & 0xBFF8_9C00) == 0x0F00_0400 {
        let imm8 = ((raw >> 5) & 0x1F) | (((raw >> 16) & 0x7) << 5);
        let shift = (((raw >> 12) & 0xF) >> 1) & 0x3;
        return Some(Instr {
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
        return Some(Instr {
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
        return Some(Instr {
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
        return Some(Instr {
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
        return Some(Instr {
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
        return Some(Instr {
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
        return Some(Instr {
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
        return Some(Instr {
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
        return Some(Instr {
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
    if (raw & 0xFFFF_FC00) == 0x4F00_0400 {
        return Some(Instr {
            op: Opcode::SimdMovi,
            rd: (raw & 0x1F) as u8,
            rn: 0,
            rm: 0,
            imm: 0,
            sf: true,
            cond: 0,
            size: 16,
        });
    }
    if let Some(instr) = decode_simd_ld_structure_multi(raw) {
        return Some(instr);
    }
    if let Some(instr) = decode_simd_st1_multi(raw) {
        return Some(instr);
    }

    let bits28_24 = (raw >> 24) & 0x1F;
    let bits28_23 = (raw >> 23) & 0x3F;
    let bits28_21 = (raw >> 21) & 0xFF;
    let bits31_26 = (raw >> 26) & 0x3F;
    let bits31_24 = (raw >> 24) & 0xFF;
    let bits31_24_masked_7e = ((raw >> 24) & 0x7E) as u32;

    // MRS/MSR decoding
    let top12 = (raw >> 20) & 0xFFF;
    if top12 == 0xD53 {
        let rd = (raw & 0x1F) as u8;
        let sysreg_id = ((raw >> 5) & 0x7FFF) as u16;
        return Some(Instr {
            op: Opcode::Mrs,
            rd,
            rn: 0,
            rm: 0,
            imm: sysreg_id as u64,
            sf: true,
            cond: 0,
            size: 0,
        });
    }
    if top12 == 0xD51 {
        let rd = (raw & 0x1F) as u8;
        let sysreg_id = ((raw >> 5) & 0x7FFF) as u16;
        return Some(Instr {
            op: Opcode::Msr,
            rd,
            rn: 0,
            rm: 0,
            imm: sysreg_id as u64,
            sf: true,
            cond: 0,
            size: 0,
        });
    }
    if (raw & 0xFFE0001F) == 0xD4000001 {
        let imm16 = ((raw >> 5) & 0xFFFF) as u64;
        return Some(Instr {
            size: 0,
            op: Opcode::Svc,
            rd: 0,
            rn: 0,
            rm: 0,
            imm: imm16,
            sf: true,
            cond: 0,
        });
    }
    if (raw & 0xFFE0001F) == 0xD4200000 {
        let imm16 = ((raw >> 5) & 0xFFFF) as u64;
        return Some(Instr {
            size: 0,
            op: Opcode::Brk,
            rd: 0,
            rn: 0,
            rm: 0,
            imm: imm16,
            sf: true,
            cond: 0,
        });
    }
    if (raw >> 24) == 0xD5 {
        match raw {
            0xD503_203F => return system::decode_yield(),
            0xD503_205F => return system::decode_wfe(),
            0xD503_207F => return system::decode_wfi(),
            0xD503_305F => return system::decode_clrex(),
            0xD503_309F | 0xD503_30BF | 0xD503_30DF | 0xD503_39BF | 0xD503_3BBF | 0xD503_3F9F
            | 0xD503_3FDF => return system::decode_barrier(),
            _ => {}
        }
        let op0 = (raw >> 19) & 0x3;
        let l = (raw >> 21) & 1;
        let crn = (raw >> 12) & 0xF;
        if l == 0 && op0 == 1 && crn == 8 {
            return system::decode_tlbi(raw);
        }
        if (raw & 0xFFFF_FFE0) == 0xD50B_7420 {
            return system::decode_dc_zva(raw);
        }
        if (raw & 0xFFFF_F01F) == 0xD503_401F {
            let daif_bits = ((raw >> 8) & 0xF) as u8;
            let op2 = (raw >> 5) & 0x7;
            let cond = match op2 {
                0b110 => 1, // DAIFSet
                0b111 => 2, // DAIFClr
                _ => 0,
            };
            if cond != 0 {
                return Some(Instr {
                    size: 0,
                    op: Opcode::Nop,
                    rd: 0,
                    rn: 0,
                    rm: 0,
                    imm: daif_bits as u64,
                    sf: true,
                    cond,
                });
            }
        }
        return system::decode_nop();
    }

    if bits28_24 == 0b10000 {
        return data_proc::decode_adr(raw);
    }
    if bits28_23 == 0b100010 {
        return data_proc::decode_addsub_imm(raw);
    }
    if bits28_23 == 0b100101 {
        let opc = (raw >> 29) & 3;
        if opc == 0 {
            return data_proc::decode_movn(raw);
        }
        if opc == 2 {
            return data_proc::decode_movz(raw);
        }
        if opc == 3 {
            return data_proc::decode_movk(raw);
        }
    }
    if bits28_23 == 0b100100 {
        return data_proc::decode_logical_imm(raw);
    }
    if bits28_23 == 0b100111 {
        return data_proc::decode_extract(raw);
    }
    if bits28_23 == 0b100110 {
        return data_proc::decode_bitfield(raw);
    }
    if bits28_21 == 0b11010100 || bits28_21 == 0b11010010 {
        return data_proc::decode_condsel(raw);
    }
    if bits28_21 == 0b11010000 {
        return data_proc::decode_addsub_carry(raw);
    }
    if bits28_21 == 0b11010110 {
        let bit30 = (raw >> 30) & 1;
        if bit30 == 1 {
            return data_proc::decode_dp_1src(raw);
        } else {
            return data_proc::decode_dp_2src(raw);
        }
    }

    let dp_reg_pat = bits28_24;
    if dp_reg_pat == 0b11010 || dp_reg_pat == 0b01011 {
        return data_proc::decode_dp_register(raw);
    }

    if bits28_24 == 0b01010 {
        return data_proc::decode_logical_reg(raw);
    }

    if let Some(instr) = ldst::decode_lse_atomic(raw) {
        return Some(instr);
    }

    let ldst_family = (raw >> 24) & 0xF8;
    if ldst_family == 0x38 || ldst_family == 0x78 || ldst_family == 0xB8 || ldst_family == 0xF8 {
        if ((raw >> 22) & 0x3FF) == 0b1111100110 {
            return system::decode_nop();
        }
        return ldst::decode_ldst(raw);
    }

    if (raw & 0x3B00_0000) == 0x1800_0000 {
        return ldst::decode_ldr_lit(raw);
    }

    let ldp_pat = (raw >> 24) & 0x1F;
    if ldp_pat & 0b11100 == 0b01000 && ldp_pat != 0b01011 {
        let is_excl = ((raw >> 29) & 1) == 0;
        if is_excl {
            return ldst::decode_ldst_excl(raw);
        } else if is_ldst_pair(raw) {
            return ldst::decode_ldst_pair(raw);
        }
    }
    if is_ldst_pair(raw) {
        return ldst::decode_ldst_pair(raw);
    }

    if bits31_26 == 0b000101 {
        return branch::decode_b(raw);
    }
    if bits31_26 == 0b100101 {
        return branch::decode_bl(raw);
    }
    if bits31_24 == 0b01010100 {
        return branch::decode_bcond(raw);
    }
    if bits31_24_masked_7e == 0b00110100 {
        return branch::decode_cbz(raw);
    }
    if bits31_24_masked_7e == 0b00110110 {
        return branch::decode_tbz(raw);
    }
    if bits31_24 == 0xD6 {
        return branch::decode_branch_reg(raw);
    }
    if bits28_24 == 0b11011 {
        return data_proc::decode_mul(raw);
    }

    None
}

fn decode_simd_ldst1_lane(raw: u32) -> Option<Instr> {
    let no_offset = (raw & 0xBFFF_0000) == 0x0D00_0000 || (raw & 0xBFFF_0000) == 0x0D40_0000;
    let post_index = (raw & 0xBFE0_0000) == 0x0D80_0000 || (raw & 0xBFE0_0000) == 0x0DC0_0000;
    if !no_offset && !post_index {
        return None;
    }

    let q = (raw >> 30) & 1;
    let load = ((raw >> 22) & 1) != 0;
    let rm_field = ((raw >> 16) & 0x1F) as u8;
    let opcode = (raw >> 13) & 0x7;
    let s = (raw >> 12) & 1;
    let size = (raw >> 10) & 0x3;
    let (element_size, lane) = match opcode {
        0b000 => (1, (q << 3) | (s << 2) | size),
        0b010 => {
            if (size & 1) != 0 {
                return None;
            }
            (2, (q << 2) | (s << 1) | (size >> 1))
        }
        0b100 => {
            if (size & 0b10) != 0 {
                return None;
            }
            if (size & 1) == 0 {
                (4, (q << 1) | s)
            } else {
                if s != 0 {
                    return None;
                }
                (8, q)
            }
        }
        _ => return None,
    };
    let rm = if post_index {
        if rm_field == 31 { 0xFE } else { rm_field }
    } else {
        0xFF
    };

    Some(Instr {
        op: if load {
            Opcode::SimdLd1Lane
        } else {
            Opcode::SimdSt1Lane
        },
        rd: (raw & 0x1F) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm,
        imm: lane as u64,
        sf: true,
        cond: element_size as u8,
        size: element_size as u8,
    })
}

fn decode_simd_ld1_multi(raw: u32) -> Option<Instr> {
    let no_offset = (raw & 0xBFFF_0000) == 0x0C40_0000;
    let post_index = (raw & 0xBFE0_0000) == 0x0CC0_0000;
    if !no_offset && !post_index {
        return None;
    }

    let register_count = match (raw >> 12) & 0xF {
        0b0010 => 4,
        0b0110 => 3,
        0b0111 => 1,
        0b1010 => 2,
        _ => return None,
    };

    let q = ((raw >> 30) & 1) as u8;
    let vector_size = if q != 0 { 16 } else { 8 };
    let rm_field = ((raw >> 16) & 0x1F) as u8;
    let (rm, imm) = if post_index {
        if rm_field == 31 {
            (0xFE, register_count as u64 * vector_size as u64)
        } else {
            (rm_field, 0)
        }
    } else {
        (0xFF, 0)
    };

    Some(Instr {
        op: if register_count == 1 {
            Opcode::SimdLd1
        } else {
            Opcode::SimdLd1Multi
        },
        rd: (raw & 0x1F) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm,
        imm,
        sf: true,
        cond: register_count,
        size: vector_size,
    })
}

fn decode_simd_st1_multi(raw: u32) -> Option<Instr> {
    let no_offset = (raw & 0xBFFF_0000) == 0x0C00_0000;
    let post_index = (raw & 0xBFE0_0000) == 0x0C80_0000;
    if !no_offset && !post_index {
        return None;
    }

    let register_count = match (raw >> 12) & 0xF {
        0b0010 => 4,
        0b0110 => 3,
        0b0111 => 1,
        0b1010 => 2,
        _ => return None,
    };

    let q = ((raw >> 30) & 1) as u8;
    let vector_size = if q != 0 { 16 } else { 8 };
    let rm_field = ((raw >> 16) & 0x1F) as u8;
    let (rm, imm) = if post_index {
        if rm_field == 31 {
            (0xFE, register_count as u64 * vector_size as u64)
        } else {
            (rm_field, 0)
        }
    } else {
        (0xFF, 0)
    };

    Some(Instr {
        op: Opcode::SimdSt1Multi,
        rd: (raw & 0x1F) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm,
        imm,
        sf: true,
        cond: register_count,
        size: vector_size,
    })
}

fn decode_simd_ld_structure_multi(raw: u32) -> Option<Instr> {
    let no_offset = (raw & 0xBFFF_0000) == 0x0C40_0000;
    let post_index = (raw & 0xBFE0_0000) == 0x0CC0_0000;
    if !no_offset && !post_index {
        return None;
    }

    let q = ((raw >> 30) & 1) as u8;
    let size = ((raw >> 10) & 0x3) as u8;
    if size == 3 && q == 0 {
        return None;
    }
    let (op, structure_count) = match (raw >> 12) & 0xF {
        0b0000 => (Opcode::SimdLd4, 4),
        0b0100 => (Opcode::SimdLd3, 3),
        0b1000 => (Opcode::SimdLd2, 2),
        _ => return None,
    };

    let vector_size = if q != 0 { 16 } else { 8 };
    let rm_field = ((raw >> 16) & 0x1F) as u8;
    let (rm, imm) = if post_index {
        if rm_field == 31 {
            (0xFE, structure_count as u64 * vector_size as u64)
        } else {
            (rm_field, 0)
        }
    } else {
        (0xFF, 0)
    };

    Some(Instr {
        op,
        rd: (raw & 0x1F) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm,
        imm,
        sf: true,
        cond: size,
        size: vector_size,
    })
}

fn decode_movi_doubleword_imm(imm8: u32) -> u64 {
    let mut value = 0u64;
    for byte in 0..8 {
        if ((imm8 >> byte) & 1) != 0 {
            value |= 0xffu64 << (byte * 8);
        }
    }
    value
}

fn decode_simd_int_fp_convert(
    raw: u32,
    scalar_pattern: u32,
    vector_pattern: u32,
    op: Opcode,
) -> Option<Instr> {
    let rd = (raw & 0x1F) as u8;
    let rn = ((raw >> 5) & 0x1F) as u8;
    let element_size = if ((raw >> 22) & 1) != 0 { 8 } else { 4 };

    if (raw & 0xFFBF_FC00) == scalar_pattern {
        return Some(Instr {
            op,
            rd,
            rn,
            rm: 0,
            imm: element_size as u64,
            sf: true,
            cond: 0,
            size: element_size,
        });
    }

    if (raw & 0xBFBF_FC00) == vector_pattern {
        let q = ((raw >> 30) & 1) != 0;
        if element_size == 8 && !q {
            return None;
        }
        return Some(Instr {
            op,
            rd,
            rn,
            rm: 0,
            imm: element_size as u64,
            sf: true,
            cond: 0,
            size: if q { 16 } else { 8 },
        });
    }

    None
}

fn decode_fp_scalar(raw: u32) -> Option<Instr> {
    let ftype = ((raw >> 22) & 0x3) as u8;
    let rd = (raw & 0x1F) as u8;
    let rn = ((raw >> 5) & 0x1F) as u8;
    let rm = ((raw >> 16) & 0x1F) as u8;

    if (raw & 0xFF3E_7C00) == 0x1E22_4000 {
        let dst_ftype = ((raw >> 15) & 0x3) as u8;
        if ftype == dst_ftype {
            return None;
        }
        let src_size = fp_scalar_type_size(ftype)?;
        let dst_size = fp_scalar_type_size(dst_ftype)?;
        let mut instr = fp_instr(Opcode::FpFcvt, rd, rn, 0, 0, dst_size);
        instr.cond = src_size;
        return Some(instr);
    }

    let size = match ftype {
        0 => 4,
        1 => 8,
        _ => return None,
    };

    if (raw & 0xFF20_0000) == 0x1F00_0000 {
        let mut instr = fp_instr(
            if ((raw >> 15) & 1) != 0 {
                Opcode::Fmsub
            } else {
                Opcode::Fmadd
            },
            rd,
            rn,
            rm,
            0,
            size,
        );
        instr.cond = ((raw >> 10) & 0x1F) as u8;
        return Some(instr);
    }
    if (raw & 0xFF20_8000) == 0x1F20_8000 {
        let mut instr = fp_instr(Opcode::Fnmsub, rd, rn, rm, 0, size);
        instr.cond = ((raw >> 10) & 0x1F) as u8;
        return Some(instr);
    }
    if (raw & 0xFF20_FC00) == 0x1E20_0800 {
        return Some(fp_instr(Opcode::FpMul, rd, rn, rm, 0, size));
    }
    if (raw & 0xFF20_FC00) == 0x1E20_8800 {
        return Some(fp_instr(Opcode::FpFnmul, rd, rn, rm, 0, size));
    }
    if (raw & 0xFF20_FC00) == 0x1E20_2800 {
        return Some(fp_instr(Opcode::FpAdd, rd, rn, rm, 0, size));
    }
    if (raw & 0xFF20_FC00) == 0x1E20_3800 {
        return Some(fp_instr(Opcode::FpSub, rd, rn, rm, 0, size));
    }
    if (raw & 0xFF20_FC00) == 0x1E20_1800 {
        return Some(fp_instr(Opcode::FpDiv, rd, rn, rm, 0, size));
    }
    if (raw & 0xFF20_FC00) == 0x1E20_6800 {
        return Some(fp_instr(Opcode::FpMaxnm, rd, rn, rm, 0, size));
    }
    if (raw & 0xFF20_FC00) == 0x1E20_7800 {
        return Some(fp_instr(Opcode::FpMinnm, rd, rn, rm, 0, size));
    }
    if (raw & 0xFFBF_FC00) == 0x1E21_4000 {
        return Some(fp_instr(Opcode::FpNeg, rd, rn, 0, 0, size));
    }
    if (raw & 0xFFBF_FC00) == 0x1E20_C000 {
        return Some(fp_instr(Opcode::FpAbs, rd, rn, 0, 0, size));
    }
    if (raw & 0xFFBF_FC00) == 0x1E21_C000 {
        return Some(fp_instr(Opcode::FpSqrt, rd, rn, 0, 0, size));
    }
    if (raw & 0xFF3F_FC00) == 0x1E25_4000 {
        return Some(fp_instr(Opcode::FpFrintm, rd, rn, 0, 0, size));
    }
    if (raw & 0xFF3F_FC00) == 0x1E24_4000 {
        return Some(fp_instr(Opcode::FpFrintn, rd, rn, 0, 0, size));
    }
    if (raw & 0xFF3F_FC00) == 0x1E26_4000 {
        return Some(fp_instr(Opcode::FpFrinta, rd, rn, 0, 0, size));
    }
    if (raw & 0xFF3F_FC00) == 0x1E27_4000 {
        return Some(fp_instr(Opcode::FpFrintx, rd, rn, 0, 0, size));
    }
    if (raw & 0xFF3F_FC00) == 0x1E24_C000 {
        return Some(fp_instr(Opcode::FpFrintp, rd, rn, 0, 0, size));
    }
    if (raw & 0xFF3F_FC00) == 0x1E25_C000 {
        return Some(fp_instr(Opcode::FpFrintz, rd, rn, 0, 0, size));
    }
    if (raw & 0xFF3F_FC00) == 0x1E27_C000 {
        return Some(fp_instr(Opcode::FpFrinti, rd, rn, 0, 0, size));
    }
    if (raw & 0xFF20_1C00) == 0x1E20_1000 {
        return Some(fp_instr(
            Opcode::FpMovImm,
            rd,
            0,
            0,
            ((raw >> 13) & 0xFF) as u64,
            size,
        ));
    }
    if (raw & 0x7FBF_FC00) == 0x1E22_0000 {
        let mut instr = fp_instr(Opcode::Scvtf, rd, rn, 0, 0, size);
        instr.sf = (raw >> 31) != 0;
        return Some(instr);
    }
    if (raw & 0x7FBF_0000) == 0x1E02_0000 {
        let scale = ((raw >> 10) & 0x3F) as u8;
        let fbits = 64u8.checked_sub(scale)?;
        let mut instr = fp_instr(Opcode::Scvtf, rd, rn, 0, fbits as u64, size);
        instr.sf = (raw >> 31) != 0;
        instr.cond = 1;
        return Some(instr);
    }
    if (raw & 0x7FBF_FC00) == 0x1E23_0000 {
        let mut instr = fp_instr(Opcode::Ucvtf, rd, rn, 0, 0, size);
        instr.sf = (raw >> 31) != 0;
        return Some(instr);
    }
    if (raw & 0x7FBF_0000) == 0x1E03_0000 {
        let scale = ((raw >> 10) & 0x3F) as u8;
        let fbits = 64u8.checked_sub(scale)?;
        let mut instr = fp_instr(Opcode::Ucvtf, rd, rn, 0, fbits as u64, size);
        instr.sf = (raw >> 31) != 0;
        instr.cond = 1;
        return Some(instr);
    }
    if (raw & 0x7FBF_0000) == 0x1E18_0000 {
        let scale = ((raw >> 10) & 0x3F) as u8;
        if (raw >> 31) == 0 && (scale & 0x20) == 0 {
            return None;
        }
        let fbits = 64u8.checked_sub(scale)?;
        let mut instr = fp_instr(Opcode::Fcvtzs, rd, rn, 0, fbits as u64, size);
        instr.sf = (raw >> 31) != 0;
        instr.cond = 1;
        return Some(instr);
    }
    if (raw & 0x7FBF_0000) == 0x1E19_0000 {
        let scale = ((raw >> 10) & 0x3F) as u8;
        if (raw >> 31) == 0 && (scale & 0x20) == 0 {
            return None;
        }
        let fbits = 64u8.checked_sub(scale)?;
        let mut instr = fp_instr(Opcode::Fcvtzu, rd, rn, 0, fbits as u64, size);
        instr.sf = (raw >> 31) != 0;
        instr.cond = 1;
        return Some(instr);
    }
    if (raw & 0x7FBF_FC00) == 0x1E20_0000 {
        let mut instr = fp_instr(Opcode::Fcvtns, rd, rn, 0, 0, size);
        instr.sf = (raw >> 31) != 0;
        return Some(instr);
    }
    if (raw & 0x7FBF_FC00) == 0x1E30_0000 {
        let mut instr = fp_instr(Opcode::Fcvtms, rd, rn, 0, 0, size);
        instr.sf = (raw >> 31) != 0;
        return Some(instr);
    }
    if (raw & 0x7FBF_FC00) == 0x1E38_0000 {
        let mut instr = fp_instr(Opcode::Fcvtzs, rd, rn, 0, 0, size);
        instr.sf = (raw >> 31) != 0;
        return Some(instr);
    }
    if (raw & 0x7FBF_FC00) == 0x1E39_0000 {
        let mut instr = fp_instr(Opcode::Fcvtzu, rd, rn, 0, 0, size);
        instr.sf = (raw >> 31) != 0;
        return Some(instr);
    }
    if (raw & 0x7FBF_FC00) == 0x1E24_0000 {
        let mut instr = fp_instr(Opcode::Fcvtas, rd, rn, 0, 0, size);
        instr.sf = (raw >> 31) != 0;
        return Some(instr);
    }
    if (raw & 0xFF20_0C00) == 0x1E20_0400 {
        let mut instr = fp_instr(
            if (raw & 0x10) != 0 {
                Opcode::Fccmpe
            } else {
                Opcode::Fccmp
            },
            0,
            rn,
            rm,
            (raw & 0xF) as u64,
            size,
        );
        instr.cond = ((raw >> 12) & 0xF) as u8;
        return Some(instr);
    }
    if (raw & 0xFF20_FC00) == 0x1E20_2000 && (raw & 0x7) == 0 {
        let cmp_kind = ((raw >> 3) & 0x3) as u8;
        let mut instr = fp_instr(
            if (cmp_kind & 0b10) != 0 {
                Opcode::Fcmpe
            } else {
                Opcode::Fcmp
            },
            0,
            rn,
            if (cmp_kind & 1) != 0 { 0 } else { rm },
            0,
            size,
        );
        instr.cond = cmp_kind & 1;
        return Some(instr);
    }
    if (raw & 0xFF20_0C00) == 0x1E20_0C00 {
        let mut instr = fp_instr(Opcode::Fcsel, rd, rn, rm, 0, size);
        instr.cond = ((raw >> 12) & 0xF) as u8;
        return Some(instr);
    }

    None
}

fn fp_instr(op: Opcode, rd: u8, rn: u8, rm: u8, imm: u64, size: u8) -> Instr {
    Instr {
        op,
        rd,
        rn,
        rm,
        imm,
        sf: size == 8,
        cond: 0,
        size,
    }
}

fn fp_scalar_type_size(ftype: u8) -> Option<u8> {
    match ftype {
        0 => Some(4),
        1 => Some(8),
        3 => Some(2),
        _ => None,
    }
}

fn decode_simd_bic_imm(raw: u32) -> Option<Instr> {
    let rd = (raw & 0x1F) as u8;
    let imm8 = (((raw >> 16) & 0x7) << 5) | ((raw >> 5) & 0x1F);
    let q = (raw >> 30) & 1;
    let cmode = (raw >> 12) & 0xF;

    if (raw & 0xBFF8_DC00) == 0x2F00_9400 {
        let shift = ((cmode >> 1) & 1) * 8;
        return Some(Instr {
            op: Opcode::SimdBicImm,
            rd,
            rn: 0,
            rm: 0,
            imm: (imm8 << shift) as u64,
            sf: true,
            cond: 2,
            size: if q == 1 { 16 } else { 8 },
        });
    }

    if (raw & 0xBFF8_9C00) == 0x2F00_1400 {
        let shift = ((cmode >> 1) & 0x3) * 8;
        return Some(Instr {
            op: Opcode::SimdBicImm,
            rd,
            rn: 0,
            rm: 0,
            imm: (imm8 << shift) as u64,
            sf: true,
            cond: 4,
            size: if q == 1 { 16 } else { 8 },
        });
    }

    None
}

fn decode_simd_shl(raw: u32) -> Option<Instr> {
    let vector = (raw & 0xBF80_FC00) == 0x0F00_5400;
    let scalar = (raw & 0xFF80_FC00) == 0x5F00_5400;
    if !vector && !scalar {
        return None;
    }

    let immh = ((raw >> 19) & 0xF) as u8;
    if immh == 0 {
        return None;
    }
    let immb = ((raw >> 16) & 0x7) as u8;
    let highest = 7 - immh.leading_zeros() as u8;
    let element_size = 1u8 << highest;
    let imm = ((immh as u16) << 3) | immb as u16;
    let element_bits = element_size as u16 * 8;
    let shift = imm.checked_sub(element_bits)? as u64;

    Some(Instr {
        op: Opcode::SimdShlImm,
        rd: (raw & 0x1F) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm: 0,
        imm: shift,
        sf: true,
        cond: element_size,
        size: if scalar {
            element_size
        } else if (raw >> 30) != 0 {
            16
        } else {
            8
        },
    })
}

fn decode_simd_sli(raw: u32) -> Option<Instr> {
    let vector = (raw & 0xBF80_FC00) == 0x2F00_5400;
    let scalar = (raw & 0xFF80_FC00) == 0x7F00_5400;
    if !vector && !scalar {
        return None;
    }

    let immh = ((raw >> 19) & 0xF) as u8;
    if immh == 0 {
        return None;
    }
    let immb = ((raw >> 16) & 0x7) as u8;
    let highest = 7 - immh.leading_zeros() as u8;
    let element_size = 1u8 << highest;
    let imm = ((immh as u16) << 3) | immb as u16;
    let element_bits = element_size as u16 * 8;
    let shift = imm.checked_sub(element_bits)? as u64;

    Some(Instr {
        op: Opcode::SimdSli,
        rd: (raw & 0x1F) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm: 0,
        imm: shift,
        sf: true,
        cond: element_size,
        size: if scalar {
            element_size
        } else if (raw >> 30) != 0 {
            16
        } else {
            8
        },
    })
}

fn decode_simd_sri(raw: u32) -> Option<Instr> {
    let vector = (raw & 0xBF80_FC00) == 0x2F00_4400;
    let scalar = (raw & 0xFF80_FC00) == 0x7F00_4400;
    if !vector && !scalar {
        return None;
    }

    let immh = ((raw >> 19) & 0xF) as u8;
    if immh == 0 {
        return None;
    }
    let q = ((raw >> 30) & 1) != 0;
    if vector && (immh & 0b1000) != 0 && !q {
        return None;
    }
    if scalar && (immh & 0b1000) == 0 {
        return None;
    }
    let immb = ((raw >> 16) & 0x7) as u8;
    let highest = 7 - immh.leading_zeros() as u8;
    let element_size = 1u8 << highest;
    let imm = ((immh as u16) << 3) | immb as u16;
    let element_bits = element_size as u16 * 8;
    let shift = (element_bits * 2).checked_sub(imm)? as u64;

    Some(Instr {
        op: Opcode::SimdSri,
        rd: (raw & 0x1F) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm: 0,
        imm: shift,
        sf: true,
        cond: element_size,
        size: if scalar {
            element_size
        } else if q {
            16
        } else {
            8
        },
    })
}

fn decode_simd_shrn(raw: u32) -> Option<Instr> {
    if (raw & 0xBF80_FC00) != 0x0F00_8400 {
        return None;
    }
    if ((raw >> 30) & 1) != 0 {
        return None;
    }

    let immh = ((raw >> 19) & 0xF) as u8;
    if immh == 0 || (immh & 0b1000) != 0 {
        return None;
    }
    let immb = ((raw >> 16) & 0x7) as u8;
    let highest = 7 - immh.leading_zeros() as u8;
    let dest_element_size = 1u8 << highest;
    let imm = ((immh as u16) << 3) | immb as u16;
    let dest_element_bits = dest_element_size as u16 * 8;
    let shift = (dest_element_bits * 2).checked_sub(imm)? as u64;

    Some(Instr {
        op: Opcode::SimdShrn,
        rd: (raw & 0x1F) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm: 0,
        imm: shift,
        sf: false,
        cond: dest_element_size,
        size: 8,
    })
}

fn decode_simd_ushr(raw: u32) -> Option<Instr> {
    let vector = (raw & 0xBF80_FC00) == 0x2F00_0400;
    let scalar = (raw & 0xFF80_FC00) == 0x7F00_0400;
    if !vector && !scalar {
        return None;
    }

    let immh = ((raw >> 19) & 0xF) as u8;
    if immh == 0 {
        return None;
    }
    let immb = ((raw >> 16) & 0x7) as u8;
    let highest = 7 - immh.leading_zeros() as u8;
    let element_size = 1u8 << highest;
    if vector && element_size == 8 && ((raw >> 30) & 1) == 0 {
        return None;
    }
    let imm = ((immh as u16) << 3) | immb as u16;
    let element_bits = element_size as u16 * 8;
    let shift = (element_bits * 2).checked_sub(imm)? as u64;

    Some(Instr {
        op: Opcode::SimdUshr,
        rd: (raw & 0x1F) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm: 0,
        imm: shift,
        sf: true,
        cond: element_size,
        size: if scalar {
            element_size
        } else if (raw >> 30) != 0 {
            16
        } else {
            8
        },
    })
}

fn decode_simd_sshr(raw: u32) -> Option<Instr> {
    if (raw & 0xFF80_FC00) == 0x5F00_0400 {
        let immh = ((raw >> 19) & 0xF) as u8;
        if (immh & 0x8) == 0 {
            return None;
        }
        let immb = ((raw >> 16) & 0x7) as u8;
        let imm = ((immh as u16) << 3) | immb as u16;
        let shift = 128u16.checked_sub(imm)? as u64;
        return Some(Instr {
            op: Opcode::SimdSshr,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: 0,
            imm: shift,
            sf: true,
            cond: 8,
            size: 8,
        });
    }

    if (raw & 0xBF80_FC00) != 0x0F00_0400 {
        return None;
    }

    let immh = ((raw >> 19) & 0xF) as u8;
    if immh == 0 {
        return None;
    }
    let immb = ((raw >> 16) & 0x7) as u8;
    let highest = 7 - immh.leading_zeros() as u8;
    let element_size = 1u8 << highest;
    if element_size == 8 && ((raw >> 30) & 1) == 0 {
        return None;
    }
    let imm = ((immh as u16) << 3) | immb as u16;
    let element_bits = element_size as u16 * 8;
    let shift = (element_bits * 2).checked_sub(imm)? as u64;

    Some(Instr {
        op: Opcode::SimdSshr,
        rd: (raw & 0x1F) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm: 0,
        imm: shift,
        sf: true,
        cond: element_size,
        size: if (raw >> 30) != 0 { 16 } else { 8 },
    })
}

fn decode_simd_ushll(raw: u32) -> Option<Instr> {
    if (raw & 0xBF80_FC00) != 0x2F00_A400 {
        return None;
    }

    let immh = ((raw >> 19) & 0xF) as u8;
    if immh == 0 {
        return None;
    }
    let immb = ((raw >> 16) & 0x7) as u8;
    let highest = 7 - immh.leading_zeros() as u8;
    let element_size = 1u8 << highest;
    if element_size > 4 {
        return None;
    }
    let imm = ((immh as u16) << 3) | immb as u16;
    let element_bits = element_size as u16 * 8;
    let shift = imm.checked_sub(element_bits)? as u64;

    Some(Instr {
        op: Opcode::SimdUshll,
        rd: (raw & 0x1F) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm: 0,
        imm: shift,
        sf: true,
        cond: element_size,
        size: 16,
    })
}

fn decode_simd_sshll(raw: u32) -> Option<Instr> {
    if (raw & 0xBF80_FC00) != 0x0F00_A400 {
        return None;
    }

    let immh = ((raw >> 19) & 0xF) as u8;
    if immh == 0 {
        return None;
    }
    let immb = ((raw >> 16) & 0x7) as u8;
    let highest = 7 - immh.leading_zeros() as u8;
    let element_size = 1u8 << highest;
    if element_size > 4 {
        return None;
    }
    let imm = ((immh as u16) << 3) | immb as u16;
    let element_bits = element_size as u16 * 8;
    let shift = imm.checked_sub(element_bits)? as u64;

    Some(Instr {
        op: Opcode::SimdSshll,
        rd: (raw & 0x1F) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm: 0,
        imm: shift,
        sf: true,
        cond: element_size,
        size: 16,
    })
}

fn decode_simd_umlal_by_element(raw: u32) -> Option<Instr> {
    if (raw & 0xBF00_F400) != 0x2F00_2000 {
        return None;
    }

    let size = ((raw >> 22) & 0x3) as u8;
    let q = ((raw >> 30) & 1) != 0;
    let l = ((raw >> 21) & 1) as u8;
    let m_bit = ((raw >> 20) & 1) as u8;
    let rm_low = ((raw >> 16) & 0xF) as u8;
    let h = ((raw >> 11) & 1) as u8;
    let (element_size, rm, index) = match size {
        0b01 => (2, rm_low, (h << 2) | (l << 1) | m_bit),
        0b10 => (4, (m_bit << 4) | rm_low, (h << 1) | l),
        _ => return None,
    };

    Some(Instr {
        op: Opcode::SimdUmlal,
        rd: (raw & 0x1F) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm,
        imm: index as u64,
        sf: q,
        cond: element_size,
        size: 16,
    })
}

fn decode_umov_element(imm5: u8) -> Option<(u8, u8)> {
    if imm5 & 0b00001 != 0 {
        Some((1, imm5 >> 1))
    } else if imm5 & 0b00010 != 0 {
        Some((2, imm5 >> 2))
    } else if imm5 & 0b00100 != 0 {
        Some((4, imm5 >> 3))
    } else if imm5 & 0b01000 != 0 {
        Some((8, imm5 >> 4))
    } else {
        None
    }
}

fn decode_simd_fp_binary(raw: u32, pattern: u32, op: Opcode) -> Option<Instr> {
    if (raw & 0xBFA0_FC00) != pattern {
        return None;
    }
    let q = ((raw >> 30) & 1) != 0;
    let element_size = if ((raw >> 22) & 1) != 0 { 8 } else { 4 };
    if element_size == 8 && !q {
        return None;
    }
    Some(Instr {
        op,
        rd: (raw & 0x1F) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm: ((raw >> 16) & 0x1F) as u8,
        imm: element_size,
        sf: true,
        cond: 0,
        size: if q { 16 } else { 8 },
    })
}

fn is_ldst_pair(raw: u32) -> bool {
    const PATTERNS: &[(u32, u32)] = &[
        (0x2800_0000, 0x7FC0_0000),
        (0x2840_0000, 0x7FC0_0000),
        (0x2880_0000, 0x7EC0_0000),
        (0x28C0_0000, 0x7EC0_0000),
        (0x2900_0000, 0x7FC0_0000),
        (0x2940_0000, 0x7FC0_0000),
        (0x2C00_0000, 0x3FC0_0000),
        (0x2C40_0000, 0x3FC0_0000),
        (0x2C80_0000, 0x3EC0_0000),
        (0x2CC0_0000, 0x3EC0_0000),
        (0x2D00_0000, 0x3FC0_0000),
        (0x2D40_0000, 0x3FC0_0000),
        (0x68C0_0000, 0xFEC0_0000),
        (0x6940_0000, 0xFFC0_0000),
    ];

    PATTERNS
        .iter()
        .any(|(opcode, mask)| (raw & mask) == *opcode)
}

#[cfg(test)]
mod tests;
