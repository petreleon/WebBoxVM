//! Load/Store instruction decoders.

use super::{Instr, Opcode};

pub(super) fn decode_ldr_lit(raw: u32) -> Option<Instr> {
    let imm19 = ((raw >> 5) & 0x7FFFF) as i32;
    let offset = (imm19 << 13) >> 11;
    let rt = (raw & 0x1F) as u8;
    Some(Instr { size: 0, op: Opcode::LdrLit, rd: rt, rn: 0, rm: 0, imm: offset as u64, sf: true, cond: 0 })
}

pub(super) fn decode_ldst_pair(raw: u32) -> Option<Instr> {
    let sf = ((raw >> 30) & 0b11) == 0b10;
    let l = ((raw >> 22) & 1) != 0;
    let op2 = ((raw >> 23) & 0x3) as u8;
    let imm7_raw = (raw >> 15) & 0x7F;
    let imm7 = if imm7_raw & 0x40 != 0 {
        (imm7_raw as i64) - 0x80
    } else {
        imm7_raw as i64
    };
    let rt2 = ((raw >> 10) & 0x1F) as u8;
    let rn = ((raw >> 5) & 0x1F) as u8;
    let rt = (raw & 0x1F) as u8;
    let v = ((raw >> 26) & 1) != 0;
    let scale = if sf { 3 } else { 2 };
    let offset = imm7 * (1i64 << scale);
    let op = if v {
        if l { Opcode::SimdLdp } else { Opcode::SimdStp }
    } else {
        if l { Opcode::Ldp } else { Opcode::Stp }
    };
    Some(Instr { size: 0, op, rd: rt, rn, rm: rt2, imm: offset as u64, sf, cond: op2 })
}

pub(super) fn decode_lse_atomic(raw: u32) -> Option<Instr> {
    let size_field = ((raw >> 30) & 3) as u8;
    let size = 1u8 << size_field;
    let rs = ((raw >> 16) & 0x1F) as u8;
    let rn = ((raw >> 5) & 0x1F) as u8;
    let rt = (raw & 0x1F) as u8;
    let sf = size == 8;

    if (raw & 0x3F20_7C00) == 0x0820_7C00 {
        if size_field <= 1 {
            let elem_size = if size_field == 0 { 4 } else { 8 };
            return Some(Instr { size: elem_size, op: Opcode::Casp, rd: rs, rn, rm: rt, imm: 0, sf: elem_size == 8, cond: 0 });
        }
        return Some(Instr { size, op: Opcode::Cas, rd: rs, rn, rm: rt, imm: 0, sf, cond: 0 });
    }

    if (raw & 0x3F20_0C00) == 0x3820_0000 {
        let atomic_op = ((raw >> 12) & 0xF) as u64;
        return Some(Instr { size, op: Opcode::Atomic, rd: rt, rn, rm: rs, imm: atomic_op, sf, cond: 0 });
    }

    None
}

pub(super) fn decode_ldst(raw: u32) -> Option<Instr> {
    let size = (raw >> 30) & 3;
    let opc = (raw >> 22) & 3;
    let rn = ((raw >> 5) & 0x1F) as u8;
    let rt = (raw & 0x1F) as u8;
    let op = match opc {
        0 => Opcode::Str,
        1 => Opcode::Ldr,
        2 | 3 => Opcode::LdrSign,
        _ => return None,
    };
    let sf = match op {
        Opcode::Str => size == 3,
        Opcode::Ldr => size == 3,
        Opcode::LdrSign => size == 2 || opc == 3,
        _ => unreachable!(),
    };

    let bit24 = (raw >> 24) & 1;
    if bit24 == 1 {
        let imm12 = ((raw >> 10) & 0xFFF) as u64;
        let shift = if size == 3 { 3 } else if size == 2 { 2 } else { size as u8 };
        return Some(Instr { size: 1u8 << size, op, rd: rt, rn, rm: 0xFF, imm: imm12 << shift, sf, cond: 0 });
    }

    let bit21 = (raw >> 21) & 1;
    let bits11_10 = (raw >> 10) & 3;

    let simm9 = || -> i64 {
        let raw9 = (raw >> 12) & 0x1FF;
        if raw9 & 0x100 != 0 { (raw9 as i64) - 0x200 } else { raw9 as i64 }
    };

    if bit21 == 0 && bits11_10 == 0b00 {
        return Some(Instr { size: 1u8 << size, op, rd: rt, rn, rm: 0xFF, imm: simm9() as u64, sf, cond: 0 });
    }
    if bit21 == 0 && bits11_10 == 0b01 {
        return Some(Instr { size: 1u8 << size, op, rd: rt, rn, rm: 0xFF, imm: simm9() as u64, sf, cond: 1 });
    }
    if bit21 == 0 && bits11_10 == 0b10 {
        return Some(Instr { size: 1u8 << size, op, rd: rt, rn, rm: 0xFF, imm: simm9() as u64, sf, cond: 0 });
    }
    if bit21 == 0 && bits11_10 == 0b11 {
        return Some(Instr { size: 1u8 << size, op, rd: rt, rn, rm: 0xFF, imm: simm9() as u64, sf, cond: 3 });
    }
    if bit21 == 1 && bits11_10 == 2 {
        let rm = ((raw >> 16) & 0x1F) as u8;
        let option = ((raw >> 13) & 7) as u8;
        let s = ((raw >> 12) & 1) as u64;
        return Some(Instr { size: 1u8 << size, op, rd: rt, rn, rm, imm: s, sf, cond: option });
    }

    None
}

pub(super) fn decode_ldst_excl(raw: u32) -> Option<Instr> {
    let size = (raw >> 30) & 3;
    let l = (raw >> 22) & 1;
    let o1 = (raw >> 23) & 1;
    let o0 = (raw >> 15) & 1;
    let pair = ((raw >> 21) & 1) != 0;
    let rs = ((raw >> 16) & 0x1F) as u8;
    let rt2 = ((raw >> 10) & 0x1F) as u8;
    let rn = ((raw >> 5) & 0x1F) as u8;
    let rt = (raw & 0x1F) as u8;

    if pair {
        let sf = size == 3;
        return if l == 1 {
            Some(Instr { op: Opcode::Ldxp, rd: rt, rn, rm: rt2, imm: 0, sf, cond: o0 as u8, size: 0 })
        } else {
            Some(Instr { op: Opcode::Stxp, rd: rt, rn, rm: rt2, imm: rs as u64, sf, cond: o0 as u8, size: 0 })
        };
    }

    if l == 1 {
        let op = if o1 == 1 && o0 == 1 && rs == 31 && rt2 == 31 { Opcode::Ldar } else { Opcode::Ldxr };
        let sz_bytes = 1 << size;
        Some(Instr { op, rd: rt, rn, rm: rt2, imm: 0, sf: size == 3, cond: o0 as u8, size: sz_bytes })
    } else {
        let op = if o1 == 1 && o0 == 1 && rt2 == 31 && rs == 31 { Opcode::Stlr } else { Opcode::Stxr };
        let sz_bytes = 1 << size;
        Some(Instr { op, rd: rt, rn, rm: rt2, imm: rs as u64, sf: size == 3, cond: o0 as u8, size: sz_bytes })
    }
}
