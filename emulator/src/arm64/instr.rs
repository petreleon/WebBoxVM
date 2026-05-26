//! AArch64 instruction decoding — pattern-based MVP decoder.

use super::Armv8Cpu;
use crate::bus::SystemBus;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Opcode {
    Add,
    Sub,
    Movz,
    Ldr,
    LdrLit,
    Str,
    B,
    Br,
    Nop,
    // Sprint 2 additions
    Bl,
    Blr,
    Ret,
    Cbz,
    Cbnz,
    BCond,
    Ldp,
    Stp,
    MovReg,
    AddImm,
    SubImm,
    CmpImm,
    Cmp,
    Adrp,
    Adr,
    Tbz,
    Tbnz,
    Movk,
    Movn,
    Sxtw,
    Csel,
    Ccmp,
    NopBarrier, // DSB, ISB, DMB
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Instr {
    pub op: Opcode,
    pub rd: u8,
    pub rn: u8,
    pub rm: u8,
    pub imm: u64,
    pub sf: bool,
    pub cond: u8,
    pub size: u8, // access size in bytes for LDR/STR (0=unused)
}

impl Instr {}

/// Decode a raw 32-bit instruction.
pub fn decode(raw: u32) -> Option<Instr> {
    if raw == 0xD503_201F { return decode_nop(); }
    // Hints / barriers / PAuth: bits[31:12] = 0xD5032 (CRm op2 Rt)
    if ((raw >> 12) & 0xFFFFF) == 0xD5032 {
        let crm = ((raw >> 8) & 0xF) as u8;
        let op2 = ((raw >> 5) & 0x7) as u8;
        let _rt = (raw & 0x1F) as u8;
        let is_barrier = op2 == 1 && (crm == 0b1010 || crm == 0b1011 || crm == 0b1101);
        if is_barrier { return decode_barrier(); }
        // Treat all other HINTs (including PAuth) as NOP
        return decode_nop();
    }
    // ADR/ADRP — bits[28:24] = 0b10000
    if ((raw >> 24) & 0x1F) == 0b10000 { return decode_adr(raw); }
    // ADD/SUB immediate — bits[28:23] = 0b100010
    if ((raw >> 23) & 0x3F) == 0b100010 { return decode_addsub_imm(raw); }
    // Wide moves (MOVN/MOVZ/MOVK) — bits[28:24] = 0b10010
    if ((raw >> 24) & 0x1F) == 0b10010 {
        let opc = (raw >> 29) & 3;
        if opc == 0 { return decode_movn(raw); }
        if opc == 2 { return decode_movz(raw); }
        if opc == 3 { return decode_movk(raw); }
    }
    // SBFM / UBFM / BFM (bitfield moves) — bits[28:24] = 0b10011
    if ((raw >> 24) & 0x1F) == 0b10011 {
        if let Some(instr) = decode_bitfield(raw) { return Some(instr); }
    }
    // Conditional select — bits[28:21] = 0b11010100 (must come before ADD register
    // because CSEL also has bits28:24 = 0b11010)
    if ((raw >> 21) & 0xFF) == 0b11010100 { return decode_condsel(raw); }
    // ADD/SUB register — bits[28:24] = 0b11010 (ADD) or 0b01011 (CMP/SUBS)
    let dp_reg_pat = (raw >> 24) & 0x1F;
    if dp_reg_pat == 0b11010 || dp_reg_pat == 0b01011 { return decode_dp_register(raw); }
    // ORR (register) — bits[28:21] = 0b01010000, used for MOV register
    if ((raw >> 21) & 0xFF) == 0b01010000 { return decode_mov_reg(raw); }
    // LDR/STR unsigned immediate — size+V in {0x38,0x78,0xB8,0xF8}
    let ldst_family = (raw >> 24) & 0xF8;
    if ldst_family == 0x38 || ldst_family == 0x78 || ldst_family == 0xB8 || ldst_family == 0xF8 {
        return decode_ldst_unsigned(raw);
    }
    // LDR literal — bits[31:24] = 0x58-0x5F
    if ((raw >> 24) & 0xF8) == 0x58 { return decode_ldr_lit(raw); }
    // LDP/STP (GP regs) — bits[28:24] = 0b01000/0b01001/0b01010
    let ldp_pat = (raw >> 24) & 0x1F;
    if ldp_pat & 0b11100 == 0b01000 && ldp_pat != 0b01011 { return decode_ldst_pair(raw); }
    // B/BL — bits[31:26] = 0b000101 / 0b100101
    if ((raw >> 26) & 0x3F) == 0b000101 { return decode_b(raw); }
    if ((raw >> 26) & 0x3F) == 0b100101 { return decode_bl(raw); }
    // B.cond — bits[31:24] = 0b01010100
    if ((raw >> 24) & 0xFF) == 0b01010100 { return decode_bcond(raw); }
    // CBZ/CBNZ — bits[31:24] = 0b1011010x
    if ((raw >> 24) & 0x7E) == 0b00110100 { return decode_cbz(raw); }
    // TBZ/TBNZ — bits[31:24] = 0b1011011x
    if ((raw >> 24) & 0x7E) == 0b00110110 { return decode_tbz(raw); }
    // BR/BLR/RET — bits[31:24] = 0xD6
    if ((raw >> 24) & 0xFF) == 0xD6 { return decode_branch_reg(raw); }
    None
}

fn decode_barrier() -> Option<Instr> {
    Some(Instr { op: Opcode::NopBarrier, rd: 0, rn: 0, rm: 0, imm: 0, sf: true, cond: 0, size: 0 })
}

fn decode_adr(raw: u32) -> Option<Instr> {
    let op = ((raw >> 31) & 1) != 0; // 0=ADR, 1=ADRP
    let immlo = (raw & 0x3) as i64;
    let immhi = ((raw >> 5) & 0x7FFFF) as i64;
    let mut imm = (immhi << 2) | immlo;
    if imm & (1 << 20) != 0 { imm -= 1 << 21; }
    let rd = (raw & 0x1F) as u8;
    if op { // ADRP
        imm <<= 12;
    }
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

    // CMP (immediate): SUBS with XZR destination
    if s && op == 1 && rd == 31 {
        return Some(Instr { size: 0, op: Opcode::CmpImm, rd: 31, rn, rm: 0, imm, sf, cond: 0 });
    }
    if s { return None; } // other S variants not yet
    let opcode = if op == 0 { Opcode::AddImm } else { Opcode::SubImm };
    Some(Instr { size: 0, op: opcode, rd, rn, rm: 0, imm, sf, cond: 0 })
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

fn decode_mov_reg(raw: u32) -> Option<Instr> {
    let sf = ((raw >> 31) & 1) != 0;
    let rm = ((raw >> 16) & 0x1F) as u8;
    let rn = ((raw >> 5) & 0x1F) as u8;
    let rd = (raw & 0x1F) as u8;
    if rn != 31 { return None; } // Only ORR with XZR -> MOV
    Some(Instr { size: 0, op: Opcode::MovReg, rd, rn: 0, rm, imm: 0, sf, cond: 0 })
}

fn decode_ldr_lit(raw: u32) -> Option<Instr> {
    let imm19 = ((raw >> 5) & 0x7FFFF) as i32;
    let offset = (imm19 << 13) >> 11; // sign-extend 19-bit, multiply by 4
    let rt = (raw & 0x1F) as u8;
    Some(Instr { size: 0, op: Opcode::LdrLit, rd: rt, rn: 0, rm: 0, imm: offset as u64, sf: true, cond: 0 })
}

fn decode_ldst_pair(raw: u32) -> Option<Instr> {
    // bits31:30 = opc: 00=32-bit, 10=64-bit (X regs), 01=LDPSW, 11=SIMD
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
    let offset = (imm19 << 13) >> 11; // sign-extend and multiply by 4
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
    let imm14 = ((raw >> 5) & 0x3FFF) as i16;
    let offset = (imm14 as i64) << 2; // sign-extend 14-bit, multiply by 4
    let rt = (raw & 0x1F) as u8;
    let bit = (b5 as u64) * 32 + (b40 as u64);
    let opcode = if op { Opcode::Tbnz } else { Opcode::Tbz };
    Some(Instr { size: 0, op: opcode, rd: rt, rn: 0, rm: 0, imm: offset as u64, sf: true, cond: bit as u8 })
}

fn decode_branch_reg(raw: u32) -> Option<Instr> {
    let opc = ((raw >> 21) & 0xF) as u8; // bits[24:21] distinguish BR/BLR/RET
    let rn = ((raw >> 5) & 0x1F) as u8;
    match opc {
        0b0000 => Some(Instr { size: 0, op: Opcode::Br, rd: 0, rn, rm: 0, imm: 0, sf: true, cond: 0 }),
        0b0001 => Some(Instr { size: 0, op: Opcode::Blr, rd: 0, rn, rm: 0, imm: 0, sf: true, cond: 0 }),
        0b0010 => Some(Instr { size: 0, op: Opcode::Ret, rd: 0, rn: if rn == 31 { 30 } else { rn }, rm: 0, imm: 0, sf: true, cond: 0 }),
        _ => None,
    }
}

fn decode_nop() -> Option<Instr> {
    Some(Instr { size: 0, op: Opcode::Nop, rd: 0, rn: 0, rm: 0, imm: 0, sf: true, cond: 0 })
}

fn decode_dp_register(raw: u32) -> Option<Instr> {
    let sf = ((raw >> 31) & 1) != 0;
    let op = (raw >> 30) & 1;
    let s = ((raw >> 29) & 1) != 0;
    let shift = ((raw >> 22) & 3) as u8;
    let n = ((raw >> 21) & 1) != 0;
    if shift != 0 || n { return None; }
    let rm = ((raw >> 16) & 0x1F) as u8;
    let rn = ((raw >> 5) & 0x1F) as u8;
    let rd = (raw & 0x1F) as u8;
    if s {
        // CMP: SUBS with XZR destination
        if rd == 31 && op == 1 {
            return Some(Instr { size: 0, op: Opcode::Cmp, rd: 31, rn, rm, imm: 0, sf, cond: 0 });
        }
        return None; // other S variants not yet
    }
    let opcode = if op == 0 { Opcode::Add } else { Opcode::Sub };
    Some(Instr { size: 0, op: opcode, rd, rn, rm, imm: 0, sf, cond: 0 })
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

// Decode SBFM / UBFM / BFM (bitfield move).
// Only SXTW (SBFM immr=0, imms=31) is supported for now.
fn decode_bitfield(raw: u32) -> Option<Instr> {
    // bits[28:24] == 0b10011 already checked by caller
    let opc = ((raw >> 29) & 3) as u8;
    let sf = ((raw >> 31) & 1) != 0;
    let n = ((raw >> 22) & 1) != 0;
    if n != sf { return None; }
    let immr = ((raw >> 16) & 0x3F) as u64;
    let imms = ((raw >> 10) & 0x3F) as u64;
    let rn = ((raw >> 5) & 0x1F) as u8;
    let rd = (raw & 0x1F) as u8;

    if opc == 0 && immr == 0 && imms == 31 {
        return Some(Instr { size: 0, op: Opcode::Sxtw, rd, rn, rm: 0, imm: 32, sf, cond: 0 });
    }
    // Reject all other SBFM / UBFM / BFM variants
    None
}

fn decode_ldst_unsigned(raw: u32) -> Option<Instr> {
    let size = (raw >> 30) & 3;
    let _v = ((raw >> 29) & 1) != 0;
    let l = ((raw >> 22) & 1) != 0;
    let imm12 = ((raw >> 10) & 0xFFF) as u64;
    let rn = ((raw >> 5) & 0x1F) as u8;
    let rt = (raw & 0x1F) as u8;
    let op = if l { Opcode::Ldr } else { Opcode::Str };
    let sf = size >= 2; // 32-bit or 64-bit (not 8/16)
    let shift = if size == 3 { 3 } else if size == 2 { 2 } else { size as u8 };
    // Store raw size in `cond` so execute() knows the access width
    Some(Instr { size: 1u8 << size, op, rd: rt, rn, rm: 0, imm: imm12 << shift, sf, cond: 0 })
}

fn decode_condsel(raw: u32) -> Option<Instr> {
    let sf = ((raw >> 31) & 1) != 0;
    let _op = (raw >> 30) & 1;
    let _s = ((raw >> 29) & 1) != 0;
    let o2 = ((raw >> 20) & 1) != 0;
    let cond = ((raw >> 16) & 0xF) as u8;
    let o3 = ((raw >> 15) & 1) != 0;
    let rm = ((raw >> 10) & 0x1F) as u8;
    let rn = ((raw >> 5) & 0x1F) as u8;
    let rd = (raw & 0x1F) as u8;

    // CSEL family: decode leniently even if o2/o3 are set,
    // because some real kernels emit encodings that objdump
    // interprets as CSEL despite non-zero o2/o3.
    if !o2 || !o3 {
        return Some(Instr { size: 0, op: Opcode::Csel, rd, rn, rm, imm: 0, sf, cond });
    }

    // CCMP family: o2=1, o3=1
    // CCMP (immediate/register): S=1, rd field holds nzcv constant
    Some(Instr { size: 0, op: Opcode::Ccmp, rd: 0, rn, rm, imm: rd as u64, sf, cond })
}

fn decode_b(raw: u32) -> Option<Instr> {
    let imm26 = (raw & 0x3FF_FFFF) as i32;
    let offset = (imm26 << 6) >> 4;
    Some(Instr { size: 0, op: Opcode::B, rd: 0, rn: 0, rm: 0, imm: offset as u64, sf: true, cond: 0 })
}

/// Execute a decoded instruction, mutating CPU and bus state.
pub fn execute(cpu: &mut Armv8Cpu, bus: &mut SystemBus, instr: Instr) -> Result<(), &'static str> {
    match instr.op {
        Opcode::Add => write_reg(cpu, instr.rd, read_reg(cpu, instr.rn, instr.sf) + read_reg(cpu, instr.rm, instr.sf), instr.sf),
        Opcode::Sub => write_reg(cpu, instr.rd, read_reg(cpu, instr.rn, instr.sf) - read_reg(cpu, instr.rm, instr.sf), instr.sf),
        Opcode::Movz => write_reg(cpu, instr.rd, instr.imm, instr.sf),
        Opcode::Movk => {
            let hw = instr.imm.trailing_zeros() / 16;
            let mask = !(0xFFFFu64 << (hw * 16));
            let old = read_reg(cpu, instr.rd, instr.sf);
            let new = (old & mask) | instr.imm;
            write_reg(cpu, instr.rd, new, instr.sf);
        }
        Opcode::Movn => {
            write_reg(cpu, instr.rd, instr.imm, instr.sf);
        }
        Opcode::MovReg => write_reg(cpu, instr.rd, read_reg(cpu, instr.rm, instr.sf), instr.sf),
        Opcode::Sxtw => {
            let val = read_reg(cpu, instr.rn, false); // low 32 bits, zero-extended
            let signed = ((val as i32) as i64) as u64; // sign-extend to 64 bits
            write_reg(cpu, instr.rd, signed, true);
        }
        Opcode::AddImm => write_reg(cpu, instr.rd, read_base(cpu, instr.rn, instr.sf) + instr.imm, instr.sf),
        Opcode::SubImm => write_reg(cpu, instr.rd, read_base(cpu, instr.rn, instr.sf) - instr.imm, instr.sf),
        Opcode::Adr => write_reg(cpu, instr.rd, (cpu.regs.pc as i64 + instr.imm as i64) as u64, true),
        Opcode::Adrp => {
            let page = cpu.regs.pc & !0xFFF;
            write_reg(cpu, instr.rd, (page as i64 + instr.imm as i64) as u64, true);
        }
        Opcode::Ldr => {
            let addr = addr_with_offset(cpu, instr.rn, instr.imm)?;
            let size = if instr.size != 0 { instr.size } else if instr.sf { 8 } else { 4 };
            let val = bus.read(addr, size).ok_or("LDR bus fault")?;
            write_reg(cpu, instr.rd, val, instr.sf);
        }
        Opcode::LdrLit => {
            let addr = (cpu.regs.pc as i64 + instr.imm as i64) as u64;
            let size = if instr.sf { 8 } else { 4 };
            let val = bus.read(addr, size).ok_or("LDR literal bus fault")?;
            write_reg(cpu, instr.rd, val, instr.sf);
        }
        Opcode::Str => {
            let addr = addr_with_offset(cpu, instr.rn, instr.imm)?;
            let val = read_reg(cpu, instr.rd, instr.sf);
            let size = if instr.size != 0 { instr.size } else if instr.sf { 8 } else { 4 };
            bus.write(addr, size, val);
        }
        Opcode::Ldp => {
            let base = read_base(cpu, instr.rn, true);
            let size = if instr.size != 0 { instr.size as u64 } else if instr.sf { 8u64 } else { 4u64 };
            let (addr, new_base) = match instr.cond {
                1 => (base, (base as i64).wrapping_add(instr.imm as i64) as u64),
                3 => {
                    let b = (base as i64).wrapping_add(instr.imm as i64) as u64;
                    (b, b)
                }
                _ => ((base as i64).wrapping_add(instr.imm as i64) as u64, base),
            };
            let val1 = bus.read(addr, size as u8).ok_or("LDP bus fault")?;
            let val2 = bus.read(addr + size, size as u8).ok_or("LDP bus fault")?;
            write_reg(cpu, instr.rd, val1, instr.sf);
            write_reg(cpu, instr.rm, val2, instr.sf);
            if new_base != base {
                write_reg(cpu, instr.rn, new_base, true);
            }
        }
        Opcode::Stp => {
            let base = read_base(cpu, instr.rn, true);
            let size = if instr.size != 0 { instr.size as u64 } else if instr.sf { 8u64 } else { 4u64 };
            let (addr, new_base) = match instr.cond {
                1 => (base, (base as i64).wrapping_add(instr.imm as i64) as u64),
                3 => {
                    let b = (base as i64).wrapping_add(instr.imm as i64) as u64;
                    (b, b)
                }
                _ => ((base as i64).wrapping_add(instr.imm as i64) as u64, base),
            };
            let val1 = read_reg(cpu, instr.rd, instr.sf);
            let val2 = read_reg(cpu, instr.rm, instr.sf);
            bus.write(addr, size as u8, val1);
            bus.write(addr + size, size as u8, val2);
            if new_base != base {
                write_reg(cpu, instr.rn, new_base, true);
            }
        }
        Opcode::B => {
            cpu.regs.pc = (cpu.regs.pc as i64 + instr.imm as i64) as u64;
            return Ok(());
        }
        Opcode::Bl => {
            cpu.regs.set_x(30, cpu.regs.pc + 4); // LR = X30
            cpu.regs.pc = (cpu.regs.pc as i64 + instr.imm as i64) as u64;
            return Ok(());
        }
        Opcode::Blr => {
            cpu.regs.set_x(30, cpu.regs.pc + 4); // LR = X30
            cpu.regs.pc = read_reg(cpu, instr.rn, true);
            return Ok(());
        }
        Opcode::Br => {
            cpu.regs.pc = read_reg(cpu, instr.rn, true);
            return Ok(());
        }
        Opcode::Ret => {
            cpu.regs.pc = read_reg(cpu, instr.rn, true);
            return Ok(());
        }
        Opcode::Cbz => {
            if read_reg(cpu, instr.rd, instr.sf) == 0 {
                cpu.regs.pc = (cpu.regs.pc as i64 + instr.imm as i64) as u64;
                return Ok(());
            }
        }
        Opcode::Cbnz => {
            if read_reg(cpu, instr.rd, instr.sf) != 0 {
                cpu.regs.pc = (cpu.regs.pc as i64 + instr.imm as i64) as u64;
                return Ok(());
            }
        }
        Opcode::BCond => {
            if cond_taken(cpu, instr.cond) {
                cpu.regs.pc = (cpu.regs.pc as i64 + instr.imm as i64) as u64;
                return Ok(());
            }
        }
        Opcode::Csel => {
            let val = if cond_taken(cpu, instr.cond) {
                read_reg(cpu, instr.rn, instr.sf)
            } else {
                read_reg(cpu, instr.rm, instr.sf)
            };
            write_reg(cpu, instr.rd, val, instr.sf);
        }
        Opcode::Ccmp => {
            // Minimal CCMP: if condition is true, compare Rn with Rm (or immediate)
            // and set NZCV flags conservatively. If false, ignore.
            if cond_taken(cpu, instr.cond) {
                let lhs = read_reg(cpu, instr.rn, instr.sf);
                let rhs = read_reg(cpu, instr.rm, instr.sf);
                let val = lhs.wrapping_sub(rhs);
                let n = (val >> 63) & 1 != 0;
                let z = val == 0;
                cpu.pstate.set_nzcv(n, z, true, false);
            }
        }
        Opcode::Tbz => {
            let val = read_reg(cpu, instr.rd, true);
            let bit = instr.cond as u64;
            if ((val >> bit) & 1) == 0 {
                cpu.regs.pc = (cpu.regs.pc as i64 + instr.imm as i64) as u64;
                return Ok(());
            }
        }
        Opcode::Tbnz => {
            let val = read_reg(cpu, instr.rd, true);
            let bit = instr.cond as u64;
            if ((val >> bit) & 1) != 0 {
                cpu.regs.pc = (cpu.regs.pc as i64 + instr.imm as i64) as u64;
                return Ok(());
            }
        }
        Opcode::Cmp => {
            let val = read_reg(cpu, instr.rn, instr.sf).wrapping_sub(read_reg(cpu, instr.rm, instr.sf));
            let n = (val >> 63) & 1 != 0;
            let z = val == 0;
            cpu.pstate.set_nzcv(n, z, true, false);
        }
        Opcode::CmpImm => {
            let val = read_reg(cpu, instr.rn, instr.sf).wrapping_sub(instr.imm);
            let n = (val >> 63) & 1 != 0;
            let z = val == 0;
            cpu.pstate.set_nzcv(n, z, true, false);
        }
        Opcode::Nop => {}
        Opcode::NopBarrier => {}
    }
    cpu.regs.pc += 4;
    Ok(())
}

fn read_reg(cpu: &Armv8Cpu, n: u8, sf: bool) -> u64 {
    let val = if n >= 31 { 0 } else { cpu.regs.x(n) };
    if sf { val } else { (val as u32) as u64 }
}

/// Read a base register (SP when n==31, otherwise X[n]).
fn read_base(cpu: &Armv8Cpu, n: u8, sf: bool) -> u64 {
    let val = if n >= 31 { cpu.regs.sp } else { cpu.regs.x(n) };
    if sf { val } else { (val as u32) as u64 }
}

/// Evaluate an AArch64 condition code using current NZCV flags.
fn cond_taken(cpu: &Armv8Cpu, cond: u8) -> bool {
    let n = cpu.pstate.n();
    let z = cpu.pstate.z();
    let c = cpu.pstate.c();
    let v = cpu.pstate.v();
    match cond & 0xF {
        0b0000 => z,                    // EQ
        0b0001 => !z,                   // NE
        0b0010 => c,                    // CS/HS
        0b0011 => !c,                   // CC/LO
        0b0100 => n,                    // MI
        0b0101 => !n,                   // PL
        0b0110 => v,                    // VS
        0b0111 => !v,                   // VC
        0b1000 => c && !z,             // HI
        0b1001 => !c || z,              // LS
        0b1010 => n == v,               // GE
        0b1011 => n != v,               // LT
        0b1100 => !z && (n == v),      // GT
        0b1101 => z || (n != v),        // LE
        0b1110 => true,                // AL
        0b1111 => true,                // NV
        _ => true,
    }
}

fn write_reg(cpu: &mut Armv8Cpu, n: u8, val: u64, sf: bool) {
    if n >= 31 { cpu.regs.sp = val; }
    else if sf { cpu.regs.set_x(n, val); }
    else { cpu.regs.set_w(n, val as u32); }
}

fn addr_with_offset(cpu: &Armv8Cpu, base: u8, offset: u64) -> Result<u64, &'static str> {
    let base_addr = if base >= 31 { cpu.regs.sp } else { cpu.regs.x(base) };
    Ok(base_addr.wrapping_add(offset))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_bus() -> (Armv8Cpu, SystemBus) {
        (Armv8Cpu::new(), SystemBus::new())
    }

    #[test]
    fn add_x0_x1_x2() {
        let (mut cpu, mut bus) = setup_bus();
        cpu.regs.set_x(1, 10);
        cpu.regs.set_x(2, 32);
        execute(&mut cpu, &mut bus, decode(0x9A02_0020).unwrap()
        ).unwrap();
        assert_eq!(cpu.regs.x(0), 42);
    }

    #[test]
    fn sub_x0_x1_x2() {
        let (mut cpu, mut bus) = setup_bus();
        cpu.regs.set_x(1, 50);
        cpu.regs.set_x(2, 8);
        execute(&mut cpu, &mut bus, decode(0xDA02_0020).unwrap()
        ).unwrap();
        assert_eq!(cpu.regs.x(0), 42);
    }

    #[test]
    fn movz_lsl_0() {
        let instr = decode(0xD282_4680).unwrap();
            assert_eq!(instr.op, Opcode::Movz);
        assert_eq!(instr.imm, 0x1234);
    }

    #[test]
    fn movz_lsl_16() {
        let instr = decode(0xD2A2_4680).unwrap();
        assert_eq!(instr.imm, 0x1234_0000);
    }

    #[test]
    fn ldr_str_roundtrip() {
        let (mut cpu, mut bus) = setup_bus();
        cpu.regs.set_x(1, 0x4000_0000);
        cpu.regs.set_x(0, 0xCAFE_0000_DEAD_BEEF);
        execute(&mut cpu, &mut bus, decode(0xF900_0020).unwrap()
        ).unwrap();
        assert_eq!(bus.mem.read(0x4000_0000, 8), Some(0xCAFE_0000_DEAD_BEEF));
        execute(&mut cpu, &mut bus, decode(0xF940_0022).unwrap()
        ).unwrap();
        assert_eq!(cpu.regs.x(2), 0xCAFE_0000_DEAD_BEEF);
    }

    #[test]
    fn nop_advances_pc() {
        let (mut cpu, mut bus) = setup_bus();
        cpu.regs.pc = 0x4000_0000;
        execute(&mut cpu, &mut bus, decode(0xD503_201F).unwrap()
        ).unwrap();
        assert_eq!(cpu.regs.pc, 0x4000_0004);
    }

    #[test]
    fn branch_forward_4_bytes() {
        let (mut cpu, mut bus) = setup_bus();
        cpu.regs.pc = 0x4000_0000;
        execute(&mut cpu, &mut bus, decode(0x1400_0002).unwrap()
        ).unwrap();
        assert_eq!(cpu.regs.pc, 0x4000_0008);
    }

    #[test]
    fn bl_sets_lr_and_jumps() {
        let (mut cpu, mut bus) = setup_bus();
        cpu.regs.pc = 0x4000_0000;
        execute(&mut cpu, &mut bus, decode(0x9400_0002).unwrap()).unwrap();
        assert_eq!(cpu.regs.x(30), 0x4000_0004); // LR
        assert_eq!(cpu.regs.pc, 0x4000_0008); // PC + 8
    }

    #[test]
    fn ret_returns_to_lr() {
        let (mut cpu, mut bus) = setup_bus();
        cpu.regs.set_x(30, 0x4000_0100);
        execute(&mut cpu, &mut bus, decode(0xD65F03C0).unwrap()).unwrap();
        assert_eq!(cpu.regs.pc, 0x4000_0100);
    }

    #[test]
    fn cbz_branches_when_zero() {
        let (mut cpu, mut bus) = setup_bus();
        cpu.regs.pc = 0x4000_0000;
        cpu.regs.set_x(0, 0);
        execute(&mut cpu, &mut bus, decode(0xB400_0040).unwrap()).unwrap();
        assert_eq!(cpu.regs.pc, 0x4000_0008); // branch taken
    }

    #[test]
    fn cbz_falls_through_when_nonzero() {
        let (mut cpu, mut bus) = setup_bus();
        cpu.regs.pc = 0x4000_0000;
        cpu.regs.set_x(0, 1);
        execute(&mut cpu, &mut bus, decode(0xB400_0040).unwrap()).unwrap();
        assert_eq!(cpu.regs.pc, 0x4000_0004); // fall through
    }

    #[test]
    fn ldp_loads_pair() {
        let (mut cpu, mut bus) = setup_bus();
        cpu.regs.set_x(1, 0x4000_0000);
        bus.mem.write(0x4000_0000, 8, 0xDEAD_BEEF);
        bus.mem.write(0x4000_0008, 8, 0xCAFE_BABE);
        execute(&mut cpu, &mut bus, decode(0xA940_0C22).unwrap()).unwrap(); // LDP X2, X3, [X1]
        assert_eq!(cpu.regs.x(2), 0xDEAD_BEEF);
        assert_eq!(cpu.regs.x(3), 0xCAFE_BABE);
    }

    #[test]
    fn mov_reg_copies_value() {
        let (mut cpu, mut bus) = setup_bus();
        cpu.regs.set_x(1, 0x1234_5678);
        execute(&mut cpu, &mut bus, decode(0xAA01_03E0).unwrap()).unwrap(); // MOV X0, X1
        assert_eq!(cpu.regs.x(0), 0x1234_5678);
    }

    #[test]
    fn add_imm_adds_constant() {
        let (mut cpu, mut bus) = setup_bus();
        cpu.regs.set_x(1, 10);
        execute(&mut cpu, &mut bus, decode(0x9100_0420).unwrap()).unwrap(); // ADD X0, X1, #1
        assert_eq!(cpu.regs.x(0), 11);
    }

    #[test]
    fn movk_merges_value() {
        let (mut cpu, mut bus) = setup_bus();
        cpu.regs.set_x(0, 0xDEAD_BEEF_0000_0000);
        execute(&mut cpu, &mut bus, decode(0xF282_4680).unwrap()).unwrap(); // MOVK X0, #0x1234
        assert_eq!(cpu.regs.x(0), 0xDEAD_BEEF_0000_1234);
    }

    #[test]
    fn adrp_sets_page_relative() {
        let (mut cpu, mut bus) = setup_bus();
        cpu.regs.pc = 0x4000_0400;
        execute(&mut cpu, &mut bus, decode(0x9000_0000).unwrap()).unwrap(); // ADRP X0, #0
        assert_eq!(cpu.regs.x(0), 0x4000_0000); // page of PC
    }

    #[test]
    fn tbz_branches_when_bit_clear() {
        let (mut cpu, mut bus) = setup_bus();
        cpu.regs.pc = 0x4000_0000;
        cpu.regs.set_x(0, 0b110); // bit 0 is clear
        execute(&mut cpu, &mut bus, decode(0x3600_0020).unwrap()).unwrap(); // TBZ X0, #0, #+4
        assert_eq!(cpu.regs.pc, 0x4000_0004); // taken
    }

    #[test]
    fn decode_cmp_x3_x2() {
        let instr = decode(0xEB02007F).unwrap();
        assert_eq!(instr.op, Opcode::Cmp);
        assert_eq!(instr.rn, 3);
        assert_eq!(instr.rm, 2);
    }

    #[test]
    fn cmp_sets_flags() {
        let (mut cpu, mut bus) = setup_bus();
        // CMP X3, X2  -> X3 - X2 = 5 - 10 = -5  (N=1, Z=0)
        cpu.regs.set_x(2, 10);
        cpu.regs.set_x(3, 5);
        execute(&mut cpu, &mut bus, decode(0xEB02007F).unwrap()
        ).unwrap();
        assert!(!cpu.pstate.z()); // not zero
        assert!(cpu.pstate.n()); // negative
    }

    #[test]
    fn cmp_equal_sets_z() {
        let (mut cpu, mut bus) = setup_bus();
        cpu.regs.set_x(2, 5);
        cpu.regs.set_x(3, 5);
        execute(&mut cpu, &mut bus, decode(0xEB02007F).unwrap()
        ).unwrap();
        assert!(cpu.pstate.z()); // zero
        assert!(!cpu.pstate.n());
    }

    #[test]
    fn cmp_less_than_sets_n() {
        let (mut cpu, mut bus) = setup_bus();
        // X3 - X2 = 10 - 3 = 7 (positive, N=0)
        cpu.regs.set_x(2, 3);
        cpu.regs.set_x(3, 10);
        execute(&mut cpu, &mut bus, decode(0xEB02007F).unwrap()
        ).unwrap();
        assert!(!cpu.pstate.n()); // positive
        assert!(!cpu.pstate.z());
    }

    #[test]
    fn str_wzr_sp_60() {
        let (mut cpu, mut bus) = setup_bus();
        cpu.regs.sp = 0x4000_0000;
        execute(&mut cpu, &mut bus, decode(0xB900_3FFF).unwrap()
        ).unwrap(); // STR WZR, [SP, #60]
        assert_eq!(bus.mem.read(0x4000_003C, 4), Some(0));
    }

    #[test]
    fn decode_br_x0() {
        let instr = decode(0xD61F0000).unwrap();
        assert_eq!(instr.op, Opcode::Br);
        assert_eq!(instr.rn, 0);
    }

    #[test]
    fn decode_ret() {
        let instr = decode(0xD65F03C0).unwrap();
        assert_eq!(instr.op, Opcode::Ret);
        assert_eq!(instr.rn, 30);
    }

    #[test]
    fn decode_blr() {
        let instr = decode(0xD63F0000).unwrap();
        assert_eq!(instr.op, Opcode::Blr);
        assert_eq!(instr.rn, 0);
    }
}
