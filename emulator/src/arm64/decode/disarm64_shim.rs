//! shim: disarm64 vs legacy cross-validator.
//!
//! Decode with both disarm64 and our legacy decoder, cross-validate the
//! opcode, and return the legacy Instr (which has proven operand extraction).
//! This gives us disarm64 correctness validation without operand extraction bugs.

use super::super::opcodes::{Instr, Opcode};
use disarm64::decoder;
#[cfg(debug_assertions)]
use std::sync::OnceLock;

/// Decode with legacy, validate against disarm64 (debug builds only).
pub fn decode(raw: u32) -> Option<Instr> {
    let legacy = super::decode_legacy(raw)?;

    // Cross-validate against disarm64 in debug builds
    #[cfg(debug_assertions)]
    if log_disarm64_mismatches() {
        if let Some(d64) = decoder::decode(raw) {
            if let Some(expected) = mnemonic_to_opcode(raw, d64.mnemonic) {
                if legacy.op != expected {
                    eprintln!(
                        "DISARM64 MISMATCH: raw=0x{raw:08x} legacy={:?} disarm64={:?}",
                        legacy.op, expected
                    );
                }
            }
        }
    }

    Some(legacy)
}

#[cfg(debug_assertions)]
fn log_disarm64_mismatches() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var_os("WEBBOXVM_DISARM64_MISMATCHES").is_some())
}

fn mnemonic_to_opcode(raw: u32, m: disarm64::decoder::Mnemonic) -> Option<Opcode> {
    use disarm64::decoder::Mnemonic as M;
    Some(match m {
        M::r#add if (raw & 0xBF20_FC00) == 0x0E20_8400 => Opcode::SimdAddVec,
        M::r#add => Opcode::Add,
        M::r#adds => Opcode::Adds,
        M::r#sub if (raw & 0xBF20_FC00) == 0x2E20_8400 => Opcode::SimdSubVec,
        M::r#sub => Opcode::Sub,
        M::r#subs => Opcode::Subs,
        M::r#adc => Opcode::Adc,
        M::r#adcs => Opcode::Adcs,
        M::r#sbc => Opcode::Sbc,
        M::r#sbcs => Opcode::Sbcs,
        M::r#movz => Opcode::Movz,
        M::r#movk => Opcode::Movk,
        M::r#movn => Opcode::Movn,
        M::r#and if (raw & 0xBFE0_FC00) == 0x0E20_1C00 => Opcode::SimdAnd,
        M::r#and => Opcode::AndReg,
        M::r#ands => Opcode::AndsReg,
        M::r#bic if (raw & 0xBFE0_FC00) == 0x0E60_1C00 => Opcode::SimdBic,
        M::r#orr if (raw & 0xBFE0_FC00) == 0x0EA0_1C00 => Opcode::SimdOrr,
        M::r#orn if (raw & 0xBFE0_FC00) == 0x0EE0_1C00 => Opcode::SimdOrn,
        M::r#bsl if (raw & 0xBFE0_FC00) == 0x2E60_1C00 => Opcode::SimdBsl,
        M::r#bit if (raw & 0xBFE0_FC00) == 0x2EA0_1C00 => Opcode::SimdBit,
        M::r#bif if (raw & 0xBFE0_FC00) == 0x2EE0_1C00 => Opcode::SimdBif,
        M::r#orr => Opcode::OrrReg,
        M::r#eor => Opcode::EorReg,
        M::r#csel => Opcode::Csel,
        M::r#csinc => Opcode::Csinc,
        M::r#csinv => Opcode::Csinv,
        M::r#csneg => Opcode::Csneg,
        M::r#ldr | M::r#ldur if ((raw >> 26) & 1) != 0 => Opcode::SimdLdr,
        M::r#ldr | M::r#ldur => Opcode::Ldr,
        M::r#str | M::r#stur if ((raw >> 26) & 1) != 0 => Opcode::SimdStr,
        M::r#str | M::r#stur => Opcode::Str,
        M::r#ldp => Opcode::Ldp,
        M::r#stp => Opcode::Stp,
        M::r#ldxr => Opcode::Ldxr,
        M::r#ldar => Opcode::Ldar,
        M::r#stxr => Opcode::Stxr,
        M::r#stlr => Opcode::Stlr,
        M::r#ldxp => Opcode::Ldxp,
        M::r#stxp => Opcode::Stxp,
        M::r#b | M::r#b_ => Opcode::B,
        M::r#bl => Opcode::Bl,
        M::r#br => Opcode::Br,
        M::r#blr => Opcode::Blr,
        M::r#ret => Opcode::Ret,
        M::r#cbz => Opcode::Cbz,
        M::r#cbnz => Opcode::Cbnz,
        M::r#tbz => Opcode::Tbz,
        M::r#tbnz => Opcode::Tbnz,
        M::r#bc_ => Opcode::BCond,
        M::r#adr => Opcode::Adr,
        M::r#adrp => Opcode::Adrp,
        M::r#mrs => Opcode::Mrs,
        M::r#svc => Opcode::Svc,
        M::r#brk => Opcode::Brk,
        M::r#eret => Opcode::Eret,
        M::r#hint => Opcode::Nop,
        M::r#mul if (raw & 0xBF20_FC00) == 0x0E20_9C00 => Opcode::SimdMulVec,
        M::r#mla if (raw & 0xBF20_FC00) == 0x0E20_9400 => Opcode::SimdMlaVec,
        M::r#madd | M::r#mul => Opcode::Madd,
        M::r#msub => Opcode::Msub,
        M::r#smulh => Opcode::Smulh,
        M::r#umulh => Opcode::Umulh,
        M::r#udiv => Opcode::Udiv,
        M::r#sdiv => Opcode::Sdiv,
        M::r#lsl | M::r#lslv => Opcode::Lslv,
        M::r#lsr | M::r#lsrv => Opcode::Lsrv,
        M::r#asr | M::r#asrv => Opcode::Asrv,
        M::r#rev => Opcode::Rev,
        M::r#rbit => Opcode::Rbit,
        M::r#clz => Opcode::Clz,
        M::r#crc32b | M::r#crc32h | M::r#crc32w | M::r#crc32x => Opcode::Crc32,
        M::r#sxtw => Opcode::Sxtw,
        M::r#ccmn => Opcode::Ccmn,
        M::r#ccmp => Opcode::Ccmp,
        M::r#fadd => Opcode::FpAdd,
        M::r#fsub => Opcode::FpSub,
        M::r#fmul => Opcode::FpMul,
        M::r#fdiv => Opcode::FpDiv,
        M::r#fneg if (raw & 0xBFBF_FC00) == 0x2EA0_F800 => Opcode::SimdFpNeg,
        M::r#fneg => Opcode::FpNeg,
        M::r#fabs => Opcode::FpAbs,
        M::r#fsqrt => Opcode::FpSqrt,
        M::r#fcvt => Opcode::FpFcvt,
        M::r#frintm => Opcode::FpFrintm,
        M::r#frintn => Opcode::FpFrintn,
        M::r#frinta => Opcode::FpFrinta,
        M::r#frintx => Opcode::FpFrintx,
        M::r#frintz => Opcode::FpFrintz,
        M::r#fmadd => Opcode::Fmadd,
        M::r#fmsub => Opcode::Fmsub,
        M::r#fnmsub => Opcode::Fnmsub,
        M::r#fcsel => Opcode::Fcsel,
        M::r#scvtf => Opcode::Scvtf,
        M::r#ucvtf => Opcode::Ucvtf,
        M::r#fcvtzs => Opcode::Fcvtzs,
        M::r#fcvtzu if (raw & 0xFFBF_FC00) == 0x7EA1_B800 => Opcode::SimdFcvtzu,
        M::r#fcvtzu => Opcode::Fcvtzu,
        M::r#fcvtas => Opcode::Fcvtas,
        M::r#fcmp => Opcode::Fcmp,
        M::r#fcmpe => Opcode::Fcmpe,
        M::r#fccmp => Opcode::Fccmp,
        M::r#fccmpe => Opcode::Fccmpe,
        M::r#fmov if (raw & 0xFFBF_FC00) == 0x1E20_4000 => Opcode::SimdFmovReg64,
        M::r#fmov if (raw & 0xFFFF_FC00) == 0x9E67_0000 => Opcode::SimdFmovGprToD,
        M::r#fmov if (raw & 0xFFFF_FC00) == 0x9E66_0000 => Opcode::SimdFmovDToGpr,
        M::r#fmov if (raw & 0x7F3F_FC00) == 0x1E27_0000 => Opcode::SimdFmovGprToS,
        M::r#fmov if (raw & 0x7F3F_FC00) == 0x1E26_0000 => Opcode::SimdFmovSToGpr,
        M::r#fmov if (raw & 0xFFFF_FC00) == 0x9EAE_0000 => Opcode::SimdFmovLaneToGpr,
        M::r#fmov if (raw & 0xFFFF_FC00) == 0x9EAF_0000 => Opcode::SimdInsGprLane,
        M::r#fmov if (raw & 0xFF20_1C00) == 0x1E20_1000 => Opcode::FpMovImm,
        M::r#umov => Opcode::SimdUmov,
        M::r#smov if simd_smov_is_valid(raw) => Opcode::SimdSmov,
        M::r#dup if (raw & 0xBFE0_FC00) == 0x0E00_0400 => Opcode::SimdDupElem,
        M::r#dup => Opcode::SimdDupByte,
        M::r#ins if (raw & 0xFFE0_8400) == 0x6E00_0400 => Opcode::SimdInsElem,
        M::r#ins if (raw & 0xFFE0_FC00) == 0x4E00_1C00 => Opcode::SimdInsGprLane,
        M::r#aese => Opcode::SimdAese,
        M::r#aesd => Opcode::SimdAesd,
        M::r#aesmc => Opcode::SimdAesmc,
        M::r#aesimc => Opcode::SimdAesimc,
        M::r#eor3 => Opcode::SimdEor3,
        M::r#bcax => Opcode::SimdBcax,
        M::r#rax1 => Opcode::SimdRax1,
        M::r#xar => Opcode::SimdXar,
        M::r#addp => Opcode::SimdAddp,
        M::r#addv => Opcode::SimdAddv,
        M::r#umaxv => Opcode::SimdUmaxv,
        M::r#cmeq if (raw & 0xBF3F_FC00) == 0x0E20_9800 => Opcode::SimdCmeqZero,
        M::r#cmeq if (raw & 0xBF20_FC00) == 0x2E20_8C00 => Opcode::SimdCmeqReg,
        M::r#cmhi if (raw & 0xBF20_FC00) == 0x2E20_3400 || (raw & 0xFFE0_FC00) == 0x7EE0_3400 => {
            Opcode::SimdCmhiReg
        }
        M::r#abs if (raw & 0xBF3F_FC00) == 0x0E20_B800 || (raw & 0xFFFF_FC00) == 0x5EE0_B800 => {
            Opcode::SimdAbs
        }
        M::r#neg if (raw & 0xFF3F_FC00) == 0x7E20_B800 => Opcode::SimdNeg,
        M::r#neg if (raw & 0xBF3F_FC00) == 0x2E20_B800 => Opcode::SimdNeg,
        M::r#ext => Opcode::SimdExt,
        M::r#cnt => Opcode::SimdCnt,
        M::r#cmtst => Opcode::SimdCmtst,
        M::r#umax if (raw & 0xBF20_FC00) == 0x2E20_6400 => Opcode::SimdUmaxVec,
        M::r#umin if (raw & 0xBF20_FC00) == 0x2E20_6C00 => Opcode::SimdUminVec,
        M::r#shl => Opcode::SimdShlImm,
        M::r#sli => Opcode::SimdSli,
        M::r#sri => Opcode::SimdSri,
        M::r#shrn => Opcode::SimdShrn,
        M::r#sshr => Opcode::SimdSshr,
        M::r#ushr => Opcode::SimdUshr,
        M::r#ushl => Opcode::SimdUshl,
        M::r#xtn => Opcode::SimdXtn,
        M::r#rev64 if (raw & 0xBF3F_FC00) == 0x0E20_0800 => Opcode::SimdRev64,
        M::r#rev32 if (raw & 0xBF3F_FC00) == 0x2E20_0800 => Opcode::SimdRev32,
        M::r#uzp1 if (raw & 0xBF20_FC00) == 0x0E00_1800 => Opcode::SimdUzp1,
        M::r#trn1 if (raw & 0xBF20_FC00) == 0x0E00_2800 => Opcode::SimdTrn1,
        M::r#zip1 if (raw & 0xBF20_FC00) == 0x0E00_3800 => Opcode::SimdZip1,
        M::r#zip2 if (raw & 0xBF20_FC00) == 0x0E00_7800 => Opcode::SimdZip2,
        M::r#tbl if (raw & 0xBFE0_9C00) == 0x0E00_0000 => Opcode::SimdTbl,
        M::r#not => Opcode::SimdNot,
        M::r#movi => Opcode::SimdMovi,
        M::r#mvni => Opcode::SimdMvni,
        M::r#ushll => Opcode::SimdUshll,
        M::r#sshll => Opcode::SimdSshll,
        M::r#shll | M::r#shll2 => Opcode::SimdShll,
        M::r#saddl | M::r#saddl2 => Opcode::SimdSaddl,
        M::r#usubl | M::r#usubl2 => Opcode::SimdUsubl,
        M::r#ssubw | M::r#ssubw2 => Opcode::SimdSsubw,
        M::r#umlal | M::r#umlal2 => Opcode::SimdUmlal,
        M::r#uqsub if (raw & 0xFF20_FC00) == 0x7E20_2C00 => Opcode::SimdUqsub,
        M::r#uminp => Opcode::SimdUminp,
        M::r#ld1r => Opcode::SimdLd1r,
        M::r#ld1 if simd_ldst1_single_lane(raw) => Opcode::SimdLd1Lane,
        M::r#ld1 if simd_ld1_multi_register_count(raw) == Some(1) => Opcode::SimdLd1,
        M::r#ld1 if simd_ld1_multi_register_count(raw).is_some() => Opcode::SimdLd1Multi,
        M::r#st1 if simd_ldst1_single_lane(raw) => Opcode::SimdSt1Lane,
        M::r#ld4 if simd_ld1_multi_register_count(raw) == Some(1) => Opcode::SimdLd1,
        M::r#ld4 if simd_ld1_multi_register_count(raw).is_some() => Opcode::SimdLd1Multi,
        M::r#ld2 if simd_ld_structure_elements(raw) == Some(2) => Opcode::SimdLd2,
        M::r#ld3 if simd_ld_structure_elements(raw) == Some(3) => Opcode::SimdLd3,
        M::r#ld4 if simd_ld_structure_elements(raw) == Some(2) => Opcode::SimdLd2,
        M::r#ld4 if simd_ld_structure_elements(raw) == Some(3) => Opcode::SimdLd3,
        M::r#ld4 if simd_ld_structure_elements(raw) == Some(4) => Opcode::SimdLd4,
        M::r#st1 if simd_st1_multi_register_count(raw).is_some() => Opcode::SimdSt1Multi,
        M::r#st4 if simd_st1_multi_register_count(raw).is_some() => Opcode::SimdSt1Multi,
        M::r#st4 if (raw & 0xFFFF_FC00) == 0x4C9F_7800 => Opcode::SimdSt4Single,
        _ => return None,
    })
}

fn simd_ld1_multi_register_count(raw: u32) -> Option<u8> {
    let no_offset = (raw & 0xBFFF_0000) == 0x0C40_0000;
    let post_index = (raw & 0xBFE0_0000) == 0x0CC0_0000;
    if !no_offset && !post_index {
        return None;
    }

    match (raw >> 12) & 0xF {
        0b0010 => Some(4),
        0b0110 => Some(3),
        0b0111 => Some(1),
        0b1010 => Some(2),
        _ => None,
    }
}

fn simd_smov_is_valid(raw: u32) -> bool {
    if (raw & 0xBFE0_FC00) != 0x0E00_2C00 {
        return false;
    }

    let q = ((raw >> 30) & 1) != 0;
    let imm5 = ((raw >> 16) & 0x1F) as u8;
    let Some((element_size, _)) = simd_move_element(imm5) else {
        return false;
    };
    let data_size = if q { 8 } else { 4 };
    (element_size as usize) < data_size
}

fn simd_move_element(imm5: u8) -> Option<(u8, u8)> {
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

fn simd_ldst1_single_lane(raw: u32) -> bool {
    let no_offset = (raw & 0xBFFF_0000) == 0x0D00_0000 || (raw & 0xBFFF_0000) == 0x0D40_0000;
    let post_index = (raw & 0xBFE0_0000) == 0x0D80_0000 || (raw & 0xBFE0_0000) == 0x0DC0_0000;
    if !no_offset && !post_index {
        return false;
    }

    matches!((raw >> 13) & 0x7, 0b000 | 0b010 | 0b100)
}

fn simd_ld_structure_elements(raw: u32) -> Option<u8> {
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

    match (raw >> 12) & 0xF {
        0b0000 => Some(4),
        0b0100 => Some(3),
        0b1000 => Some(2),
        _ => None,
    }
}

fn simd_st1_multi_register_count(raw: u32) -> Option<u8> {
    let no_offset = (raw & 0xBFFF_0000) == 0x0C00_0000;
    let post_index = (raw & 0xBFE0_0000) == 0x0C80_0000;
    if !no_offset && !post_index {
        return None;
    }

    match (raw >> 12) & 0xF {
        0b0010 => Some(4),
        0b0110 => Some(3),
        0b0111 => Some(1),
        0b1010 => Some(2),
        _ => None,
    }
}
