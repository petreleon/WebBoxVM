//! shim: disarm64 vs legacy cross-validator.
//!
//! Decode with both disarm64 and our legacy decoder, cross-validate the
//! opcode, and return the legacy Instr (which has proven operand extraction).
//! This gives us disarm64 correctness validation without operand extraction bugs.

use disarm64::decoder;
use super::super::opcodes::{Instr, Opcode};

/// Decode with legacy, validate against disarm64 (debug builds only).
pub fn decode(raw: u32) -> Option<Instr> {
    let legacy = super::decode_legacy(raw)?;

    // Cross-validate against disarm64 in debug builds
    #[cfg(debug_assertions)]
    if let Some(d64) = decoder::decode(raw) {
        if let Some(expected) = mnemonic_to_opcode(d64.mnemonic) {
            if legacy.op != expected {
                eprintln!("DISARM64 MISMATCH: raw=0x{raw:08x} legacy={:?} disarm64={:?}", legacy.op, expected);
            }
        }
    }

    Some(legacy)
}

fn mnemonic_to_opcode(m: disarm64::decoder::Mnemonic) -> Option<Opcode> {
    use disarm64::decoder::Mnemonic::*;
    Some(match m {
        r#add => Opcode::Add,
        r#adds => Opcode::Adds,
        r#sub => Opcode::Sub,
        r#subs => Opcode::Subs,
        r#movz => Opcode::Movz,
        r#movk => Opcode::Movk,
        r#movn => Opcode::Movn,
        r#and => Opcode::AndReg,
        r#ands => Opcode::AndsReg,
        r#orr => Opcode::OrrReg,
        r#eor => Opcode::EorReg,
        r#csel => Opcode::Csel,
        r#csinc => Opcode::Csinc,
        r#csinv => Opcode::Csinv,
        r#csneg => Opcode::Csneg,
        r#ldr | r#ldur => Opcode::Ldr,
        r#str | r#stur => Opcode::Str,
        r#ldp => Opcode::Ldp,
        r#stp => Opcode::Stp,
        r#ldxr => Opcode::Ldxr,
        r#ldar => Opcode::Ldar,
        r#stxr => Opcode::Stxr,
        r#stlr => Opcode::Stlr,
        r#ldxp => Opcode::Ldxp,
        r#stxp => Opcode::Stxp,
        r#b | r#b_ => Opcode::B,
        r#bl => Opcode::Bl,
        r#br => Opcode::Br,
        r#blr => Opcode::Blr,
        r#ret => Opcode::Ret,
        r#cbz => Opcode::Cbz,
        r#cbnz => Opcode::Cbnz,
        r#tbz => Opcode::Tbz,
        r#tbnz => Opcode::Tbnz,
        r#bc_ => Opcode::BCond,
        r#adr => Opcode::Adr,
        r#adrp => Opcode::Adrp,
        r#mrs => Opcode::Mrs,
        r#svc => Opcode::Svc,
        r#brk => Opcode::Brk,
        r#eret => Opcode::Eret,
        r#nop => Opcode::Nop,
        r#wfi => Opcode::Wfi,
        r#wfe => Opcode::Wfe,
        r#madd | r#mul => Opcode::Madd,
        r#msub => Opcode::Msub,
        r#smulh => Opcode::Smulh,
        r#umulh => Opcode::Umulh,
        r#udiv => Opcode::Udiv,
        r#sdiv => Opcode::Sdiv,
        r#lsl | r#lslv => Opcode::Lslv,
        r#lsr | r#lsrv => Opcode::Lsrv,
        r#asr | r#asrv => Opcode::Asrv,
        r#rev => Opcode::Rev,
        r#rbit => Opcode::Rbit,
        r#clz => Opcode::Clz,
        r#sxtw => Opcode::Sxtw,
        r#tlbi => Opcode::Tlbi,
        r#ccmp => Opcode::Ccmp,
        _ => return None,
    })
}
