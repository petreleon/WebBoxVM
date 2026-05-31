//! shim: disarm64 vs legacy cross-validator.
//!
//! Decode with both disarm64 and our legacy decoder, cross-validate the
//! opcode, and return the legacy Instr (which has proven operand extraction).
//! This gives us disarm64 correctness validation without operand extraction bugs.

use super::super::opcodes::{Instr, Opcode};
use disarm64::decoder;

/// Decode with legacy, validate against disarm64 (debug builds only).
pub fn decode(raw: u32) -> Option<Instr> {
    let legacy = super::decode_legacy(raw)?;

    // Cross-validate against disarm64 in debug builds
    #[cfg(debug_assertions)]
    if let Some(d64) = decoder::decode(raw) {
        if let Some(expected) = mnemonic_to_opcode(d64.mnemonic) {
            if legacy.op != expected {
                eprintln!(
                    "DISARM64 MISMATCH: raw=0x{raw:08x} legacy={:?} disarm64={:?}",
                    legacy.op, expected
                );
            }
        }
    }

    Some(legacy)
}

fn mnemonic_to_opcode(m: disarm64::decoder::Mnemonic) -> Option<Opcode> {
    use disarm64::decoder::Mnemonic as M;
    Some(match m {
        M::r#add => Opcode::Add,
        M::r#adds => Opcode::Adds,
        M::r#sub => Opcode::Sub,
        M::r#subs => Opcode::Subs,
        M::r#movz => Opcode::Movz,
        M::r#movk => Opcode::Movk,
        M::r#movn => Opcode::Movn,
        M::r#and => Opcode::AndReg,
        M::r#ands => Opcode::AndsReg,
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
        _ => return None,
    })
}
