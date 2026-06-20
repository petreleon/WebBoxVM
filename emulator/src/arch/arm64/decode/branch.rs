//! Branch instruction decoders.

use super::{Instr, Opcode};

pub(super) fn decode_bl(raw: u32) -> Option<Instr> {
    let imm26 = ((raw & 0x3FF_FFFF) as i64) << 38 >> 38;
    Some(Instr {
        size: 0,
        op: Opcode::Bl,
        rd: 0,
        rn: 0,
        rm: 0,
        imm: (imm26 << 2) as u64,
        sf: true,
        cond: 0,
    })
}

pub(super) fn decode_bcond(raw: u32) -> Option<Instr> {
    let imm19 = ((raw >> 5) & 0x7FFFF) as i32;
    let offset = (imm19 << 13) >> 11;
    let cond = (raw & 0xF) as u8;
    Some(Instr {
        size: 0,
        op: Opcode::BCond,
        rd: 0,
        rn: 0,
        rm: 0,
        imm: offset as u64,
        sf: true,
        cond,
    })
}

pub(super) fn decode_cbz(raw: u32) -> Option<Instr> {
    let sf = ((raw >> 31) & 1) != 0;
    let op = ((raw >> 24) & 1) != 0;
    let imm19 = ((raw >> 5) & 0x7FFFF) as i32;
    let offset = (imm19 << 13) >> 11;
    let rt = (raw & 0x1F) as u8;
    let opcode = if op { Opcode::Cbnz } else { Opcode::Cbz };
    Some(Instr {
        size: 0,
        op: opcode,
        rd: rt,
        rn: 0,
        rm: 0,
        imm: offset as u64,
        sf,
        cond: 0,
    })
}

pub(super) fn decode_tbz(raw: u32) -> Option<Instr> {
    let b5 = ((raw >> 31) & 1) as u8;
    let op = ((raw >> 24) & 1) != 0;
    let b40 = ((raw >> 19) & 0x1F) as u8;
    let mut imm14 = ((raw >> 5) & 0x3FFF) as i16;
    if imm14 & 0x2000 != 0 {
        imm14 -= 0x4000;
    }
    let offset = (imm14 as i64) << 2;
    let rt = (raw & 0x1F) as u8;
    let bit = (b5 as u64) * 32 + (b40 as u64);
    let opcode = if op { Opcode::Tbnz } else { Opcode::Tbz };
    let sf = ((raw >> 31) & 1) != 0;
    Some(Instr {
        size: 0,
        op: opcode,
        rd: rt,
        rn: 0,
        rm: 0,
        imm: offset as u64,
        sf,
        cond: bit as u8,
    })
}

pub(super) fn decode_branch_reg(raw: u32) -> Option<Instr> {
    if matches!(raw, 0xD69F_03E0 | 0xD69F_0BFF | 0xD69F_0FFF) {
        return Some(branch_reg_instr(Opcode::Eret, 0));
    }
    if let Some(op) = decode_pac_branch_reg(raw) {
        return Some(branch_reg_instr(op, ((raw >> 5) & 0x1F) as u8));
    }
    let opc = ((raw >> 21) & 0xF) as u8;
    let rn = ((raw >> 5) & 0x1F) as u8;
    let (op, rn) = match opc {
        0b0000 => (Opcode::Br, rn),
        0b0001 => (Opcode::Blr, rn),
        0b0010 => (Opcode::Ret, if rn == 31 { 30 } else { rn }),
        _ => return None,
    };
    Some(branch_reg_instr(op, rn))
}

fn decode_pac_branch_reg(raw: u32) -> Option<Opcode> {
    match raw & 0xFFFF_FC00 {
        0xD71F_0800 | 0xD71F_0C00 => Some(Opcode::Br),
        0xD73F_0800 | 0xD73F_0C00 => Some(Opcode::Blr),
        _ => None,
    }
}

fn branch_reg_instr(op: Opcode, rn: u8) -> Instr {
    Instr {
        size: 0,
        op,
        rd: 0,
        rn,
        rm: 0,
        imm: 0,
        sf: true,
        cond: 0,
    }
}

pub(super) fn decode_b(raw: u32) -> Option<Instr> {
    let imm26 = (raw & 0x3FF_FFFF) as u64;
    let offset = (((imm26 << 38) as i64) >> 38) << 2;
    Some(Instr {
        size: 0,
        op: Opcode::B,
        rd: 0,
        rn: 0,
        rm: 0,
        imm: offset as u64,
        sf: true,
        cond: 0,
    })
}
