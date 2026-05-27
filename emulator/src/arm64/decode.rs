//! AArch64 instruction decoder (pattern-based).

use super::opcodes::{Instr, Opcode};
use super::bitmask_imm::decode_bitmask_imm;

/// Decode a raw 32-bit word into an instruction.
pub fn decode(raw: u32) -> Option<Instr> {
    if raw == 0xD503_201F { return decode_nop(); }

    let bits28_24 = (raw >> 24) & 0x1F;
    let bits28_23 = (raw >> 23) & 0x3F;
    let bits28_21 = (raw >> 21) & 0xFF;
    let bits31_26 = (raw >> 26) & 0x3F;
    let bits31_24 = (raw >> 24) & 0xFF;
    let bits31_24_masked_7e = ((raw >> 24) & 0x7E) as u32;

    // Hints / barriers / PAuth: bits[31:12] = 0xD5032
    if ((raw >> 12) & 0xFFFFF) == 0xD5032 {
        let crm = ((raw >> 8) & 0xF) as u8;
        let op2 = ((raw >> 5) & 0x7) as u8;
        let is_barrier = op2 == 1 && (crm == 0b1010 || crm == 0b1011 || crm == 0b1101);
        if is_barrier { return decode_barrier(); }
        return decode_nop();
    }

    // MRS/MSR decoding
    let top12 = (raw >> 20) & 0xFFF;
    if top12 == 0xD53 {
        // MRS Xt, sysreg
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
        // MSR sysreg, Xt
        let rd = (raw & 0x1F) as u8; // Rt source
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
    if (raw >> 24) == 0xD5 {
        let op0 = (raw >> 19) & 0x3;
        let l = (raw >> 21) & 1;
        let crn = (raw >> 12) & 0xF;
        if l == 0 && op0 == 1 && crn == 8 {
            return decode_tlbi(raw);
        }
        return decode_nop(); // Remaining system / cache maintenance instructions → NOP
    }

    if bits28_24 == 0b10000 { return decode_adr(raw); }
    if bits28_23 == 0b100010 { return decode_addsub_imm(raw); }
    if bits28_23 == 0b100101 {
        let opc = (raw >> 29) & 3;
        if opc == 0 { return decode_movn(raw); }
        if opc == 2 { return decode_movz(raw); }
        if opc == 3 { return decode_movk(raw); }
    }
    // Logical-immediate: AND, ORR, EOR, ANDS (N = bit22 = 0)
    if bits28_23 == 0b100100 {
        return decode_logical_imm(raw);
    }
    if bits28_24 == 0b10011 { return decode_bitfield(raw); }
    if bits28_21 == 0b11010100 || bits28_21 == 0b11010010 { return decode_condsel(raw); }

    // ADD/SUB register — 0b11010 or 0b01011
    let dp_reg_pat = bits28_24;
    if dp_reg_pat == 0b11010 || dp_reg_pat == 0b01011 {
        return decode_dp_register(raw);
    }

    if bits28_24 == 0b01010 { return decode_logical_reg(raw); }

    // LDR/STR unsigned immediate — size+V in {0x38,0x78,0xB8,0xF8}
    let ldst_family = (raw >> 24) & 0xF8;
    if ldst_family == 0x38 || ldst_family == 0x78 || ldst_family == 0xB8 || ldst_family == 0xF8 {
        return decode_ldst_unsigned(raw);
    }

    // LDR literal
    if ((raw >> 24) & 0xF8) == 0x58 { return decode_ldr_lit(raw); }

    // LDP/STP
    let ldp_pat = (raw >> 24) & 0x1F;
    if ldp_pat & 0b11100 == 0b01000 && ldp_pat != 0b01011 {
        return decode_ldst_pair(raw);
    }

    if bits31_26 == 0b000101 { return decode_b(raw); }
    if bits31_26 == 0b100101 { return decode_bl(raw); }
    if bits31_24 == 0b01010100 { return decode_bcond(raw); }
    if bits31_24_masked_7e == 0b00110100 { return decode_cbz(raw); }
    if bits31_24_masked_7e == 0b00110110 { return decode_tbz(raw); }
    if bits31_24 == 0xD6 { return decode_branch_reg(raw); }
    if bits28_24 == 0b11011 { return decode_mul(raw); }

    None
}

fn decode_nop() -> Option<Instr> {
    Some(Instr { op: Opcode::Nop, rd: 0, rn: 0, rm: 0, imm: 0, sf: true, cond: 0, size: 0 })
}

fn decode_tlbi(raw: u32) -> Option<Instr> {
    let op1 = ((raw >> 16) & 0x7) as u8;
    let crm = ((raw >> 8) & 0xF) as u8;
    let op2 = ((raw >> 5) & 0x7) as u8;
    let rt = (raw & 0x1F) as u8;
    let variant = ((op1 as u64) << 16) | ((crm as u64) << 8) | ((op2 as u64) << 4) | (rt as u64);
    Some(Instr { op: Opcode::Tlbi, rd: 0, rn: 0, rm: 0, imm: variant, sf: true, cond: 0, size: 0 })
}

fn decode_barrier() -> Option<Instr> {
    Some(Instr { op: Opcode::NopBarrier, rd: 0, rn: 0, rm: 0, imm: 0, sf: true, cond: 0, size: 0 })
}

fn decode_adr(raw: u32) -> Option<Instr> {
    let op = ((raw >> 31) & 1) != 0; // 0=ADR, 1=ADRP
    let immlo = ((raw >> 29) & 0x3) as i64;
    let immhi = ((raw >> 5) & 0x7FFFF) as i64;
    let mut imm = (immhi << 2) | immlo;
    if imm & (1 << 20) != 0 { imm -= 1 << 21; }
    let rd = (raw & 0x1F) as u8;
    if op { imm <<= 12; }
    Some(Instr { size: 0, op: if op { Opcode::Adrp } else { Opcode::Adr }, rd, rn: 0, rm: 0, imm: imm as u64, sf: true, cond: 0 })
}

fn decode_addsub_imm(raw: u32) -> Option<Instr> {
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

fn decode_movz(raw: u32) -> Option<Instr> {
    let sf = ((raw >> 31) & 1) != 0;
    if ((raw >> 29) & 3) != 2 { return None; }
    let hw = ((raw >> 21) & 3) as u64;
    if hw > (if sf { 3 } else { 1 }) { return None; }
    let imm16 = ((raw >> 5) & 0xFFFF) as u64;
    let rd = (raw & 0x1F) as u8;
    Some(Instr { size: 0, op: Opcode::Movz, rd, rn: 0, rm: 0, imm: imm16 << (hw * 16), sf, cond: 0 })
}

fn decode_movk(raw: u32) -> Option<Instr> {
    let sf = ((raw >> 31) & 1) != 0;
    if ((raw >> 29) & 3) != 3 { return None; }
    let hw = ((raw >> 21) & 3) as u64;
    if hw > (if sf { 3 } else { 1 }) { return None; }
    let imm16 = ((raw >> 5) & 0xFFFF) as u64;
    let rd = (raw & 0x1F) as u8;
    Some(Instr { size: 0, op: Opcode::Movk, rd, rn: 0, rm: 0, imm: imm16 << (hw * 16), sf, cond: 0 })
}

fn decode_movn(raw: u32) -> Option<Instr> {
    let sf = ((raw >> 31) & 1) != 0;
    if ((raw >> 29) & 3) != 0 { return None; }
    let hw = ((raw >> 21) & 3) as u64;
    if hw > (if sf { 3 } else { 1 }) { return None; }
    let imm16 = ((raw >> 5) & 0xFFFF) as u64;
    let rd = (raw & 0x1F) as u8;
    let val = !(imm16 << (hw * 16));
    Some(Instr { size: 0, op: Opcode::Movn, rd, rn: 0, rm: 0, imm: if sf { val } else { val & 0xFFFF_FFFF }, sf, cond: 0 })
}

fn decode_logical_reg(raw: u32) -> Option<Instr> {
    let sf = ((raw >> 31) & 1) != 0;
    let opc = (raw >> 30) & 3;
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

fn decode_ldr_lit(raw: u32) -> Option<Instr> {
    let imm19 = ((raw >> 5) & 0x7FFFF) as i32;
    let offset = (imm19 << 13) >> 11; // sign-extend 19-bit, multiply by 4
    let rt = (raw & 0x1F) as u8;
    Some(Instr { size: 0, op: Opcode::LdrLit, rd: rt, rn: 0, rm: 0, imm: offset as u64, sf: true, cond: 0 })
}

fn decode_ldst_pair(raw: u32) -> Option<Instr> {
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
    let scale = if sf { 3 } else { 2 };
    let offset = imm7 * (1i64 << scale);
    let op = if l { Opcode::Ldp } else { Opcode::Stp };
    Some(Instr { size: 0, op, rd: rt, rn, rm: rt2, imm: offset as u64, sf, cond: op2 })
}

fn decode_bl(raw: u32) -> Option<Instr> {
    let imm26 = (raw & 0x3FF_FFFF) as i32;
    let offset = (imm26 << 6) >> 4; // sign-extend and multiply by 4
    Some(Instr { size: 0, op: Opcode::Bl, rd: 0, rn: 0, rm: 0, imm: offset as u64, sf: true, cond: 0 })
}

fn decode_bcond(raw: u32) -> Option<Instr> {
    let imm19 = ((raw >> 5) & 0x7FFFF) as i32;
    let offset = (imm19 << 13) >> 11;
    let cond = (raw & 0xF) as u8;
    Some(Instr { size: 0, op: Opcode::BCond, rd: 0, rn: 0, rm: 0, imm: offset as u64, sf: true, cond })
}

fn decode_cbz(raw: u32) -> Option<Instr> {
    let sf = ((raw >> 31) & 1) != 0;
    let op = ((raw >> 24) & 1) != 0; // 0=CBZ, 1=CBNZ
    let imm19 = ((raw >> 5) & 0x7FFFF) as i32;
    let offset = (imm19 << 13) >> 11;
    let rt = (raw & 0x1F) as u8;
    let opcode = if op { Opcode::Cbnz } else { Opcode::Cbz };
    Some(Instr { size: 0, op: opcode, rd: rt, rn: 0, rm: 0, imm: offset as u64, sf, cond: 0 })
}

fn decode_tbz(raw: u32) -> Option<Instr> {
    let b5 = ((raw >> 31) & 1) as u8;
    let op = ((raw >> 24) & 1) != 0; // 0=TBZ, 1=TBNZ
    let b40 = ((raw >> 19) & 0x1F) as u8;
    let mut imm14 = ((raw >> 5) & 0x3FFF) as i16;
    if imm14 & 0x2000 != 0 {
        imm14 -= 0x4000;
    }
    let offset = (imm14 as i64) << 2;
    let rt = (raw & 0x1F) as u8;
    let bit = (b5 as u64) * 32 + (b40 as u64);
    let opcode = if op { Opcode::Tbnz } else { Opcode::Tbz };
    Some(Instr { size: 0, op: opcode, rd: rt, rn: 0, rm: 0, imm: offset as u64, sf: true, cond: bit as u8 })
}

fn decode_branch_reg(raw: u32) -> Option<Instr> {
    let opc = ((raw >> 21) & 0xF) as u8;
    let rn = ((raw >> 5) & 0x1F) as u8;
    match opc {
        0b0000 => Some(Instr { size: 0, op: Opcode::Br, rd: 0, rn, rm: 0, imm: 0, sf: true, cond: 0 }),
        0b0001 => Some(Instr { size: 0, op: Opcode::Blr, rd: 0, rn, rm: 0, imm: 0, sf: true, cond: 0 }),
        0b0010 => Some(Instr { size: 0, op: Opcode::Ret, rd: 0, rn: if rn == 31 { 30 } else { rn }, rm: 0, imm: 0, sf: true, cond: 0 }),
        _ => None,
    }
}

fn decode_dp_register(raw: u32) -> Option<Instr> {
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
        // Add/subtract (extended register)
        if shift != 0 { return None; }
        let option = (imm6 >> 3) & 7;
        let imm3 = imm6 & 7;
        if s { return None; } // CMP/CMN (extended register) not yet supported
        let opcode = if op == 0 { Opcode::AddExt } else { Opcode::SubExt };
        return Some(Instr { size: 0, op: opcode, rd, rn, rm, imm: imm3 as u64, sf, cond: option });
    }

    if s {
        if rd == 31 && op == 1 {
            return Some(Instr { size: 0, op: Opcode::Cmp, rd: 31, rn, rm, imm: imm6 as u64, sf, cond: shift });
        }
        return None;
    }
    let opcode = if op == 0 { Opcode::Add } else { Opcode::Sub };
    Some(Instr { size: 0, op: opcode, rd, rn, rm, imm: imm6 as u64, sf, cond: shift })
}

fn decode_bitfield(raw: u32) -> Option<Instr> {
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

fn decode_logical_imm(raw: u32) -> Option<Instr> {
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

fn decode_ldst_unsigned(raw: u32) -> Option<Instr> {
    let size = (raw >> 30) & 3;
    let l = ((raw >> 22) & 1) != 0;
    let imm12 = ((raw >> 10) & 0xFFF) as u64;
    let rn = ((raw >> 5) & 0x1F) as u8;
    let rt = (raw & 0x1F) as u8;
    let op = if l { Opcode::Ldr } else { Opcode::Str };
    let sf = size >= 2;
    let shift = if size == 3 { 3 } else if size == 2 { 2 } else { size as u8 };
    Some(Instr { size: 1u8 << size, op, rd: rt, rn, rm: 0, imm: imm12 << shift, sf, cond: 0 })
}

fn decode_condsel(raw: u32) -> Option<Instr> {
    let sf = ((raw >> 31) & 1) != 0;
    let op = (raw >> 30) & 1;
    let o2 = ((raw >> 10) & 1) != 0;
    let cond = if op == 1 { ((raw >> 12) & 0xF) as u8 } else { ((raw >> 12) & 0xF) as u8 };
    let o3 = ((raw >> 11) & 1) != 0;
    let _rm = ((raw >> 16) & 0x1F) as u8;
    let rn = ((raw >> 5) & 0x1F) as u8;
    let rd = (raw & 0x1F) as u8;

    if op == 1 {
        // CCMP or CCMN
        let is_register = ((raw >> 21) & 3) == 3; // bits22:21 = 11
        let is_immediate = ((raw >> 21) & 3) == 2; // bits22:21 = 10
        if !is_register && !is_immediate { return None; }
        let nzcv = (raw & 0xF) as u64;
        let rm_or_imm = ((raw >> 16) & 0x1F) as u64;
        return Some(Instr { size: 0, op: Opcode::Ccmp, rd: rd, rn, rm: rm_or_imm as u8, imm: nzcv, sf, cond });
    }

    if !o2 || !o3 {
        return Some(Instr { size: 0, op: Opcode::Csel, rd, rn, rm: _rm, imm: 0, sf, cond });
    }
    None
}

fn decode_mul(raw: u32) -> Option<Instr> {
    let bits31_29 = (raw >> 29) & 0x7;
    let op54 = (raw >> 21) & 0x7;
    let o0 = ((raw >> 15) & 1) != 0;
    let rd = (raw & 0x1F) as u8;
    let rn = ((raw >> 5) & 0x1F) as u8;
    let ra = ((raw >> 10) & 0x1F) as u8;
    let rm = ((raw >> 16) & 0x1F) as u8;

    let (sf, size) = match bits31_29 {
        0b000 => {
            if op54 == 0b000 {
                (false, 0) // W-variant normal
            } else {
                return None;
            }
        }
        0b100 => {
            match op54 {
                0b000 => (true, 0),  // X-variant normal
                0b001 => (true, 2),  // SMADDL / SMSUBL
                0b101 => (true, 1),  // UMADDL / UMSUBL
                _ => return None,
            }
        }
        _ => return None,
    };

    let op = if o0 { Opcode::Msub } else { Opcode::Madd };
    Some(Instr {
        op,
        rd,
        rn,
        rm,
        imm: 0,
        sf,
        cond: ra,
        size,
    })
}

fn decode_b(raw: u32) -> Option<Instr> {
    let imm26 = (raw & 0x3FF_FFFF) as i32;
    let offset = (imm26 << 6) >> 4;
    Some(Instr { size: 0, op: Opcode::B, rd: 0, rn: 0, rm: 0, imm: offset as u64, sf: true, cond: 0 })
}

#[cfg(test)]
mod tests;
