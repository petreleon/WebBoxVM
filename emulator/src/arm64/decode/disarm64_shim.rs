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
        M::r#orr if (raw & 0xBFE0_FC00) == 0x0EA0_1C00 => Opcode::SimdOrr,
        M::r#orr => Opcode::OrrReg,
        M::r#eor => Opcode::EorReg,
        M::r#csel => Opcode::Csel,
        M::r#csinc => Opcode::Csinc,
        M::r#csinv => Opcode::Csinv,
        M::r#csneg => Opcode::Csneg,
        M::r#ldr | M::r#ldur => Opcode::Ldr,
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
        M::r#sxtw => Opcode::Sxtw,
        M::r#ccmn => Opcode::Ccmn,
        M::r#ccmp => Opcode::Ccmp,
        M::r#fadd => Opcode::FpAdd,
        M::r#fsub => Opcode::FpSub,
        M::r#fmul => Opcode::FpMul,
        M::r#fdiv => Opcode::FpDiv,
        M::r#fneg => Opcode::FpNeg,
        M::r#fcsel => Opcode::Fcsel,
        M::r#scvtf => Opcode::Scvtf,
        M::r#fcvtzs => Opcode::Fcvtzs,
        M::r#fcmp => Opcode::Fcmp,
        M::r#fcmpe => Opcode::Fcmpe,
        M::r#fmov if (raw & 0xFFFF_FC00) == 0x1E60_4000 => Opcode::SimdFmovReg64,
        M::r#fmov if (raw & 0xFFFF_FC00) == 0x9E67_0000 => Opcode::SimdFmovGprToD,
        M::r#fmov if (raw & 0xFFFF_FC00) == 0x9E66_0000 => Opcode::SimdFmovDToGpr,
        M::r#fmov if (raw & 0x7F3F_FC00) == 0x1E27_0000 => Opcode::SimdFmovGprToS,
        M::r#fmov if (raw & 0x7F3F_FC00) == 0x1E26_0000 => Opcode::SimdFmovSToGpr,
        M::r#fmov if (raw & 0xFFFF_FC00) == 0x9EAE_0000 => Opcode::SimdFmovLaneToGpr,
        M::r#fmov if (raw & 0xFF20_FC00) == 0x1E20_1000 => Opcode::FpMovImm,
        M::r#umov => Opcode::SimdUmov,
        M::r#dup if (raw & 0xBFE0_FC00) == 0x0E00_0400 => Opcode::SimdDupElem,
        M::r#dup => Opcode::SimdDupByte,
        M::r#ins if (raw & 0xFFE0_8400) == 0x6E00_0400 => Opcode::SimdInsElem,
        M::r#ins if (raw & 0xFFE0_FC00) == 0x4E00_1C00 => Opcode::SimdInsGprLane,
        M::r#addp => Opcode::SimdAddp,
        M::r#addv => Opcode::SimdAddv,
        M::r#ext => Opcode::SimdExt,
        M::r#cnt => Opcode::SimdCnt,
        M::r#cmtst => Opcode::SimdCmtst,
        M::r#shl => Opcode::SimdShlImm,
        M::r#sli => Opcode::SimdSli,
        M::r#ushr => Opcode::SimdUshr,
        M::r#ushl => Opcode::SimdUshl,
        M::r#xtn => Opcode::SimdXtn,
        M::r#rev32 if (raw & 0xBF3F_FC00) == 0x2E20_0800 => Opcode::SimdRev32,
        M::r#uzp1 if (raw & 0xBF20_FC00) == 0x0E00_1800 => Opcode::SimdUzp1,
        M::r#not => Opcode::SimdNot,
        M::r#movi => Opcode::SimdMovi,
        M::r#mvni => Opcode::SimdMvni,
        M::r#ushll => Opcode::SimdUshll,
        M::r#uminp => Opcode::SimdUminp,
        M::r#ld1r => Opcode::SimdLd1r,
        M::r#ld4 if (raw & 0xBFFF_F000) == 0x0C40_A000 => Opcode::SimdLd1Multi,
        M::r#st4 if (raw & 0xBFFF_F000) == 0x0C00_A000 => Opcode::SimdSt1Multi,
        M::r#st4 if (raw & 0xFFFF_FC00) == 0x4C9F_7800 => Opcode::SimdSt4Single,
        _ => return None,
    })
}
