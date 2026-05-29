//! Data-processing instruction decoders: ADR, add/sub, move, logical, bitfield, DP, condsel, multiply.

use crate::arm64::bitmask_imm::decode_bitmask_imm;
use super::{Instr, Opcode};

pub(super) fn decode_adr(raw: u32) -> Option<Instr> {
    let op = ((raw >> 31) & 1) != 0;
    let immlo = ((raw >> 29) & 0x3) as i64;
    let immhi = ((raw >> 5) & 0x7FFFF) as i64;
    let mut imm = (immhi << 2) | immlo;
    if imm & (1 << 20) != 0 { imm -= 1 << 21; }
    let rd = (raw & 0x1F) as u8;
    if op { imm <<= 12; }
    Some(Instr { size: 0, op: if op { Opcode::Adrp } else { Opcode::Adr }, rd, rn: 0, rm: 0, imm: imm as u64, sf: true, cond: 0 })
}

pub(super) fn decode_addsub_imm(raw: u32) -> Option<Instr> {
    let sf = ((raw >> 31) & 1) != 0;
    let op = (raw >> 30) & 1;
    let s = ((raw >> 29) & 1) != 0;
    let sh = ((raw >> 22) & 1) != 0;
    let imm12 = ((raw >> 10) & 0xFFF) as u64;
    let rn = ((raw >> 5) & 0x1F) as u8;
    let rd = (raw & 0x1F) as u8;
    let imm = if sh { imm12 << 12 } else { imm12 };

    if s {
        if op == 1 && rd == 31 {
            return Some(Instr { size: 0, op: Opcode::CmpImm, rd: 31, rn, rm: 0, imm, sf, cond: 0 });
        }
        let opcode = if op == 0 { Opcode::AddsImm } else { Opcode::SubsImm };
        return Some(Instr { size: 0, op: opcode, rd, rn, rm: 0, imm, sf, cond: 0 });
    }
    let opcode = if op == 0 { Opcode::AddImm } else { Opcode::SubImm };
    Some(Instr { size: 0, op: opcode, rd, rn, rm: 0, imm, sf, cond: 0 })
}

pub(super) fn decode_movz(raw: u32) -> Option<Instr> {
    let sf = ((raw >> 31) & 1) != 0;
    if ((raw >> 29) & 3) != 2 { return None; }
    let hw = ((raw >> 21) & 3) as u64;
    if hw > (if sf { 3 } else { 1 }) { return None; }
    let imm16 = ((raw >> 5) & 0xFFFF) as u64;
    let rd = (raw & 0x1F) as u8;
    Some(Instr { size: 0, op: Opcode::Movz, rd, rn: 0, rm: 0, imm: imm16 << (hw * 16), sf, cond: 0 })
}

pub(super) fn decode_movk(raw: u32) -> Option<Instr> {
    let sf = ((raw >> 31) & 1) != 0;
    if ((raw >> 29) & 3) != 3 { return None; }
    let hw = ((raw >> 21) & 3) as u8;
    if hw > (if sf { 3 } else { 1 }) { return None; }
    let imm16 = ((raw >> 5) & 0xFFFF) as u64;
    let rd = (raw & 0x1F) as u8;
    Some(Instr { size: 0, op: Opcode::Movk, rd, rn: 0, rm: 0, imm: imm16 << (hw as u64 * 16), sf, cond: hw })
}

pub(super) fn decode_movn(raw: u32) -> Option<Instr> {
    let sf = ((raw >> 31) & 1) != 0;
    if ((raw >> 29) & 3) != 0 { return None; }
    let hw = ((raw >> 21) & 3) as u64;
    if hw > (if sf { 3 } else { 1 }) { return None; }
    let imm16 = ((raw >> 5) & 0xFFFF) as u64;
    let rd = (raw & 0x1F) as u8;
    let val = !(imm16 << (hw * 16));
    Some(Instr { size: 0, op: Opcode::Movn, rd, rn: 0, rm: 0, imm: if sf { val } else { val & 0xFFFF_FFFF }, sf, cond: 0 })
}

pub(super) fn decode_logical_reg(raw: u32) -> Option<Instr> {
    let sf = ((raw >> 31) & 1) != 0;
    let opc = (raw >> 29) & 3;
    let shift = ((raw >> 22) & 3) as u8;
    let n = ((raw >> 21) & 1) != 0;
    let rm = ((raw >> 16) & 0x1F) as u8;
    let imm6 = ((raw >> 10) & 0x3F) as u8;
    let rn = ((raw >> 5) & 0x1F) as u8;
    let rd = (raw & 0x1F) as u8;

    if rn == 31 && opc == 1 && shift == 0 && !n && imm6 == 0 {
        return Some(Instr { size: 0, op: Opcode::MovReg, rd, rn: 0, rm, imm: 0, sf, cond: 0 });
    }

    let op = match opc {
        0 => Opcode::AndReg,
        1 => Opcode::OrrReg,
        2 => Opcode::EorReg,
        3 => Opcode::AndsReg,
        _ => return None,
    };

    let cond = ((n as u8) << 2) | shift;
    Some(Instr { size: 0, op, rd, rn, rm, imm: imm6 as u64, sf, cond })
}

pub(super) fn decode_dp_register(raw: u32) -> Option<Instr> {
    let sf = ((raw >> 31) & 1) != 0;
    let op = (raw >> 30) & 1;
    let s = ((raw >> 29) & 1) != 0;
    let shift = ((raw >> 22) & 3) as u8;
    let n = ((raw >> 21) & 1) != 0;
    let rm = ((raw >> 16) & 0x1F) as u8;
    let imm6 = ((raw >> 10) & 0x3F) as u8;
    let rn = ((raw >> 5) & 0x1F) as u8;
    let rd = (raw & 0x1F) as u8;

    if n {
        if shift != 0 { return None; }
        let option = (imm6 >> 3) & 7;
        let imm3 = imm6 & 7;
        if s && op == 1 && rd == 31 {
            return Some(Instr { size: 0, op: Opcode::Cmp, rd: 31, rn, rm, imm: imm3 as u64, sf, cond: option });
        }
        let opcode = if s {
            if op == 0 { Opcode::AddsExt } else { Opcode::SubsExt }
        } else {
            if op == 0 { Opcode::AddExt } else { Opcode::SubExt }
        };
        return Some(Instr { size: 0, op: opcode, rd, rn, rm, imm: imm3 as u64, sf, cond: option });
    }

    if s {
        if op == 1 && rd == 31 {
            return Some(Instr { size: 0, op: Opcode::Cmp, rd: 31, rn, rm, imm: imm6 as u64, sf, cond: shift });
        }
        let opcode = if op == 0 { Opcode::Adds } else { Opcode::Subs };
        return Some(Instr { size: 0, op: opcode, rd, rn, rm, imm: imm6 as u64, sf, cond: shift });
    }
    let opcode = if op == 0 { Opcode::Add } else { Opcode::Sub };
    Some(Instr { size: 0, op: opcode, rd, rn, rm, imm: imm6 as u64, sf, cond: shift })
}

pub(super) fn decode_dp_1src(raw: u32) -> Option<Instr> {
    let sf = ((raw >> 31) & 1) != 0;
    let opcode2 = ((raw >> 16) & 0x1F) as u8;
    let opcode = ((raw >> 10) & 0x3F) as u8;
    let rn = ((raw >> 5) & 0x1F) as u8;
    let rd = (raw & 0x1F) as u8;

    if opcode2 == 0 {
        match opcode {
            0b000000 => Some(Instr { size: 0, op: Opcode::Rbit, rd, rn, rm: 0, imm: 0, sf, cond: 0 }),
            0b000001 => Some(Instr { size: 0, op: Opcode::Rev16, rd, rn, rm: 0, imm: 0, sf, cond: 0 }),
            0b000010 => {
                let op = if sf { Opcode::Rev32 } else { Opcode::Rev };
                Some(Instr { size: 0, op, rd, rn, rm: 0, imm: 0, sf, cond: 0 })
            }
            0b000011 => Some(Instr { size: 0, op: Opcode::Rev, rd, rn, rm: 0, imm: 0, sf, cond: 0 }),
            0b000100 => Some(Instr { size: 0, op: Opcode::Clz, rd, rn, rm: 0, imm: 0, sf, cond: 0 }),
            _ => None,
        }
    } else {
        None
    }
}

pub(super) fn decode_dp_2src(raw: u32) -> Option<Instr> {
    let sf = ((raw >> 31) & 1) != 0;
    let rm = ((raw >> 16) & 0x1F) as u8;
    let opcode = ((raw >> 10) & 0x3F) as u8;
    let rn = ((raw >> 5) & 0x1F) as u8;
    let rd = (raw & 0x1F) as u8;

    match opcode {
        0b000010 => Some(Instr { size: 0, op: Opcode::Udiv, rd, rn, rm, imm: 0, sf, cond: 0 }),
        0b000011 => Some(Instr { size: 0, op: Opcode::Sdiv, rd, rn, rm, imm: 0, sf, cond: 0 }),
        0b001000 => Some(Instr { size: 0, op: Opcode::Lslv, rd, rn, rm, imm: 0, sf, cond: 0 }),
        0b001001 => Some(Instr { size: 0, op: Opcode::Lsrv, rd, rn, rm, imm: 0, sf, cond: 0 }),
        0b001010 => Some(Instr { size: 0, op: Opcode::Asrv, rd, rn, rm, imm: 0, sf, cond: 0 }),
        0b001011 => Some(Instr { size: 0, op: Opcode::Rorv, rd, rn, rm, imm: 0, sf, cond: 0 }),
        _ => None,
    }
}

pub(super) fn decode_bitfield(raw: u32) -> Option<Instr> {
    let opc = ((raw >> 29) & 3) as u8;
    let sf = ((raw >> 31) & 1) != 0;
    let n = ((raw >> 22) & 1) != 0;
    if n != sf { return None; }
    let immr = ((raw >> 16) & 0x3F) as u8;
    let imms = ((raw >> 10) & 0x3F) as u8;
    let rn = ((raw >> 5) & 0x1F) as u8;
    let rd = (raw & 0x1F) as u8;

    if opc == 0 && immr == 0 && imms == 31 {
        return Some(Instr { size: 0, op: Opcode::Sxtw, rd, rn, rm: 0, imm: 32, sf, cond: 0 });
    }

    let op = match opc {
        0 => Opcode::Sbfm,
        1 => Opcode::Bfm,
        2 => Opcode::Ubfm,
        _ => return None,
    };

    Some(Instr { size: 0, op, rd, rn, rm: immr, imm: imms as u64, sf, cond: 0 })
}

pub(super) fn decode_logical_imm(raw: u32) -> Option<Instr> {
    let sf = ((raw >> 31) & 1) != 0;
    let opc = (raw >> 29) & 0x3;
    let n = (raw >> 22) & 1;
    let immr = ((raw >> 16) & 0x3F) as u32;
    let imms = ((raw >> 10) & 0x3F) as u32;
    let rn = ((raw >> 5) & 0x1F) as u8;
    let rd = (raw & 0x1F) as u8;

    let imm = decode_bitmask_imm(n, immr, imms, sf)?;
    let op = match opc {
        0 => Opcode::AndImm,
        1 => Opcode::OrrImm,
        2 => Opcode::EorImm,
        3 => Opcode::AndsImm,
        _ => return None,
    };
    Some(Instr { size: 0, op, rd, rn, rm: 0, imm, sf, cond: 0 })
}

pub(super) fn decode_condsel(raw: u32) -> Option<Instr> {
    let sf = ((raw >> 31) & 1) != 0;
    let op = (raw >> 30) & 1;
    let o2 = ((raw >> 10) & 1) != 0;
    let cond = ((raw >> 12) & 0xF) as u8;
    let o3 = ((raw >> 11) & 1) != 0;
    let _rm = ((raw >> 16) & 0x1F) as u8;
    let rn = ((raw >> 5) & 0x1F) as u8;
    let rd = (raw & 0x1F) as u8;
    let bits22_21 = (raw >> 21) & 3;

    if bits22_21 == 0 {
        let opcode = match (op, o2, o3) {
            (0, false, false) => Opcode::Csel,
            (0, true, false) => Opcode::Csinc,
            (1, false, false) => Opcode::Csinv,
            (1, true, false) => Opcode::Csneg,
            _ => return None,
        };
        return Some(Instr { size: 0, op: opcode, rd, rn, rm: _rm, imm: 0, sf, cond });
    }

    let is_register = bits22_21 == 3;
    let is_immediate = bits22_21 == 2;
    if !is_register && !is_immediate { return None; }
    let nzcv = (raw & 0xF) as u64;
    let rm_or_imm = ((raw >> 16) & 0x1F) as u64;
    Some(Instr { size: 0, op: Opcode::Ccmp, rd, rn, rm: rm_or_imm as u8, imm: nzcv, sf, cond })
}

pub(super) fn decode_mul(raw: u32) -> Option<Instr> {
    let bits31_29 = (raw >> 29) & 0x7;
    let op54 = (raw >> 21) & 0x7;
    let o0 = ((raw >> 15) & 1) != 0;
    let rd = (raw & 0x1F) as u8;
    let rn = ((raw >> 5) & 0x1F) as u8;
    let ra = ((raw >> 10) & 0x1F) as u8;
    let rm = ((raw >> 16) & 0x1F) as u8;

    let (sf, size) = match bits31_29 {
        0b000 => {
            if op54 == 0b000 { (false, 0) } else { return None; }
        }
        0b100 => {
            match op54 {
                0b000 => (true, 0),
                0b001 => (true, 2),
                0b101 => (true, 1),
                0b010 => { return Some(Instr { op: Opcode::Smulh, rd, rn, rm, imm: 0, sf: true, cond: 0, size: 0 }); }
                0b110 => { return Some(Instr { op: Opcode::Umulh, rd, rn, rm, imm: 0, sf: true, cond: 0, size: 0 }); }
                _ => return None,
            }
        }
        _ => return None,
    };

    let op = if o0 { Opcode::Msub } else { Opcode::Madd };
    Some(Instr { op, rd, rn, rm, imm: 0, sf, cond: ra, size })
}
