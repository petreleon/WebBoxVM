//! shim: disarm64::decoder → our Instr conversion (thin wrapper).

use disarm64::decoder;
use super::super::opcodes::{Instr, Opcode};

pub fn decode(raw: u32) -> Option<Instr> {
    if let Some(d64) = decoder::decode(raw) {
        if let Some(our_op) = quick_map(d64.mnemonic) {
            return extract_instr(raw, our_op);
        }
    }
    // Fall back to our hand-rolled decoder
    super::decode_legacy(raw)
}

fn quick_map(m: disarm64::decoder::Mnemonic) -> Option<Opcode> {
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

fn extract_instr(raw: u32, op: Opcode) -> Option<Instr> {
    let sf = (raw >> 31) & 1 != 0;
    let rd = (raw & 0x1F) as u8;
    let rn = ((raw >> 5) & 0x1F) as u8;
    let rm = ((raw >> 16) & 0x1F) as u8;
    let imm12 = ((raw >> 10) & 0xFFF) as u64;

    let imm = match op {
        Opcode::B | Opcode::Bl => { let v = (raw & 0x03FF_FFFF) as i32; (v << 6 >> 4) as u64 }
        Opcode::BCond | Opcode::LdrLit | Opcode::Cbz | Opcode::Cbnz => {
            let v = ((raw >> 5) & 0x7FFFF) as i32; (v << 13 >> 11) as u64
        }
        Opcode::Tbz | Opcode::Tbnz => {
            let v = ((raw >> 5) & 0x3FFF) as i16;
            (if v & 0x2000 != 0 { v - 0x4000 } else { v as i16 }) as i64 as u64
        }
        Opcode::Adr | Opcode::Adrp => {
            let immlo = ((raw >> 29) & 3) as i64;
            let immhi = ((raw >> 5) & 0x7FFFF) as i64;
            let mut v = (immhi << 2) | immlo;
            if v & (1 << 20) != 0 { v -= 1 << 21; }
            if op == Opcode::Adrp { v <<= 12; }
            v as u64
        }
        Opcode::Movz | Opcode::Movn | Opcode::Movk => {
            let hw = ((raw >> 21) & 3) as u64;
            let i16 = ((raw >> 5) & 0xFFFF) as u64;
            i16 << (hw * 16)
        }
        Opcode::Mrs | Opcode::Msr => ((raw >> 5) & 0x7FFF) as u64,
        Opcode::Svc | Opcode::Brk => ((raw >> 5) & 0xFFFF) as u64,
        Opcode::Stp | Opcode::Ldp => {
            let v = ((raw >> 15) & 0x7F) as i64;
            let s = if v & 0x40 != 0 { v - 0x80 } else { v };
            (s * (1i64 << if sf { 3 } else { 2 })) as u64
        }
        _ => imm12,
    };

    let cond = match op {
        Opcode::BCond => (raw & 0xF) as u8,
        Opcode::Csel | Opcode::Csinc | Opcode::Csinv | Opcode::Csneg | Opcode::Ccmp => ((raw >> 12) & 0xF) as u8,
        Opcode::Tbz | Opcode::Tbnz => {
            let b5 = ((raw >> 31) & 1) as u8;
            let b40 = ((raw >> 19) & 0x1F) as u8;
            b5 * 32 + b40
        }
        Opcode::Movk => ((raw >> 21) & 3) as u8,
        _ => 0,
    };

    let size = ((raw >> 30) & 3) as u8;
    Some(Instr { op, rd, rn, rm, imm, sf, cond, size })
}
