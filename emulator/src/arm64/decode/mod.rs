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
    if (raw & 0xFFFF_FC00) == 0x4C40_7000 {
        return Some(Instr {
            op: Opcode::SimdLd1,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: 0xFF,
            imm: 0,
            sf: true,
            cond: 0,
            size: 16,
        });
    }
    if (raw & 0xFFFF_F000) == 0x4CDF_7000 {
        return Some(Instr {
            op: Opcode::SimdLd1,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: 0xFF,
            imm: 16,
            sf: true,
            cond: 1,
            size: 16,
        });
    }
    if (raw & 0xFFFF_FC00) == 0x4D40_8400 {
        return Some(Instr {
            op: Opcode::SimdLd1Lane,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: 0xFF,
            imm: 1,
            sf: true,
            cond: 0,
            size: 8,
        });
    }
    if (raw & 0xBFFF_F000) == 0x0D40_C000 {
        let element_size = 1u8 << (((raw >> 10) & 0x3) as u8);
        return Some(Instr {
            op: Opcode::SimdLd1r,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: 0xFF,
            imm: 0,
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
    if (raw & 0xFFFF_FC00) == 0x1E60_4000 {
        return Some(Instr {
            op: Opcode::SimdFmovReg64,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: 0,
            imm: 0,
            sf: true,
            cond: 0,
            size: 8,
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
    if (raw & 0xBF3F_FC00) == 0x0E20_9800 {
        let element_size = 1u64 << ((raw >> 22) & 0x3);
        return Some(Instr {
            op: Opcode::SimdCmeqZero,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: 0,
            imm: element_size,
            sf: true,
            cond: 0,
            size: if (raw >> 30) != 0 { 16 } else { 8 },
        });
    }
    if (raw & 0xFFE0_FC00) == 0x6E20_8C00 {
        return Some(Instr {
            op: Opcode::SimdCmeqReg,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: ((raw >> 16) & 0x1F) as u8,
            imm: 0,
            sf: true,
            cond: 0,
            size: 16,
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
    if (raw & 0xBF20_FC00) == 0x2E20_4400 {
        let element_size = 1u64 << ((raw >> 22) & 0x3);
        return Some(Instr {
            op: Opcode::SimdUshl,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: ((raw >> 16) & 0x1F) as u8,
            imm: element_size,
            sf: true,
            cond: 0,
            size: if (raw >> 30) != 0 { 16 } else { 8 },
        });
    }
    if (raw & 0xBF20_FC00) == 0x0E00_1800 {
        let element_size = 1u64 << ((raw >> 22) & 0x3);
        return Some(Instr {
            op: Opcode::SimdUzp1,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: ((raw >> 16) & 0x1F) as u8,
            imm: element_size,
            sf: true,
            cond: 0,
            size: if (raw >> 30) != 0 { 16 } else { 8 },
        });
    }
    if (raw & 0xFFFF_FC00) == 0x0F0C_8400 {
        return Some(Instr {
            op: Opcode::SimdShrn,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: 0,
            imm: 4,
            sf: false,
            cond: 0,
            size: 8,
        });
    }
    if let Some(instr) = decode_simd_shl(raw) {
        return Some(instr);
    }
    if let Some(instr) = decode_simd_sli(raw) {
        return Some(instr);
    }
    if let Some(instr) = decode_simd_ushr(raw) {
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
        let element_size = 1u64 << ((raw >> 22) & 0x3);
        return Some(Instr {
            op: Opcode::SimdAddp,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: ((raw >> 16) & 0x1F) as u8,
            imm: element_size,
            sf: true,
            cond: 0,
            size: if (raw >> 30) != 0 { 16 } else { 8 },
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
    if (raw & 0xFFE0_FC00) == 0x6EA0_1C00 {
        return Some(Instr {
            op: Opcode::SimdBit,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: ((raw >> 16) & 0x1F) as u8,
            imm: 0,
            sf: true,
            cond: 0,
            size: 16,
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
    if (raw & 0xFFFF_FC00) == 0x6F00_E400 {
        return Some(Instr {
            op: Opcode::SimdMovi,
            rd: (raw & 0x1F) as u8,
            rn: 0,
            rm: 0,
            imm: 0,
            sf: true,
            cond: 8,
            size: 16,
        });
    }
    if (raw & 0xFFFF_FC00) == 0x2F00_E400 {
        return Some(Instr {
            op: Opcode::SimdMovi,
            rd: (raw & 0x1F) as u8,
            rn: 0,
            rm: 0,
            imm: 0,
            sf: true,
            cond: 8,
            size: 8,
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
    if (raw & 0xBFFF_F000) == 0x0C40_A000 {
        return Some(Instr {
            op: Opcode::SimdLd1Multi,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: 0xFF,
            imm: 0,
            sf: true,
            cond: 2,
            size: if (raw >> 30) != 0 { 16 } else { 8 },
        });
    }
    if (raw & 0xBFFF_F000) == 0x0C00_A000 {
        return Some(Instr {
            op: Opcode::SimdSt1Multi,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: 0xFF,
            imm: 0,
            sf: true,
            cond: 2,
            size: if (raw >> 30) != 0 { 16 } else { 8 },
        });
    }
    if (raw & 0xBFFF_F000) == 0x0C9F_A000 {
        return Some(Instr {
            op: Opcode::SimdSt1Multi,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: 0xFE,
            imm: 32,
            sf: true,
            cond: 2,
            size: if (raw >> 30) != 0 { 16 } else { 8 },
        });
    }
    if (raw & 0xFFFF_FC00) == 0x4C9F_7800 {
        return Some(Instr {
            op: Opcode::SimdSt4Single,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: 0xFE,
            imm: 16,
            sf: true,
            cond: 2,
            size: 16,
        });
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
            0xD503_309F | 0xD503_30BF | 0xD503_30DF | 0xD503_3F9F | 0xD503_3FDF => {
                return system::decode_barrier();
            }
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

    if ((raw >> 24) & 0xF8) == 0x58 {
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

fn decode_fp_scalar(raw: u32) -> Option<Instr> {
    let ftype = ((raw >> 22) & 0x3) as u8;
    let size = match ftype {
        0 => 4,
        1 => 8,
        _ => return None,
    };
    let rd = (raw & 0x1F) as u8;
    let rn = ((raw >> 5) & 0x1F) as u8;
    let rm = ((raw >> 16) & 0x1F) as u8;

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
    if (raw & 0xFF20_FC00) == 0x1E20_2800 {
        return Some(fp_instr(Opcode::FpAdd, rd, rn, rm, 0, size));
    }
    if (raw & 0xFF20_FC00) == 0x1E20_3800 {
        return Some(fp_instr(Opcode::FpSub, rd, rn, rm, 0, size));
    }
    if (raw & 0xFF20_FC00) == 0x1E20_1800 {
        return Some(fp_instr(Opcode::FpDiv, rd, rn, rm, 0, size));
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
    if (raw & 0xFF3E_7C00) == 0x1E22_4000 {
        let dst_ftype = ((raw >> 15) & 0x3) as u8;
        if ftype == dst_ftype || dst_ftype > 1 {
            return None;
        }
        let dst_size = if dst_ftype == 0 { 4 } else { 8 };
        let mut instr = fp_instr(Opcode::FpFcvt, rd, rn, 0, 0, dst_size);
        instr.cond = size;
        return Some(instr);
    }
    if (raw & 0xFF3F_FC00) == 0x1E25_4000 {
        return Some(fp_instr(Opcode::FpFrintm, rd, rn, 0, 0, size));
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
