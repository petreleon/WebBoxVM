use super::*;

pub(super) fn decode(raw: u32) -> DecodeStep {
    if let Some(instr) = decode_immediate(raw) {
        return DecodeStep::Hit(instr);
    }
    if let Some(instr) = decode_indexed(raw) {
        return DecodeStep::Hit(instr);
    }
    DecodeStep::Miss
}

fn decode_immediate(raw: u32) -> Option<Instr> {
    if (raw & 0xFF3F_C000) != 0x2538_C000 {
        return None;
    }
    let size = 1u8 << (((raw >> 22) & 0x3) as u8);
    let shift = ((raw >> 13) & 1) != 0;
    if size == 1 && shift {
        return None;
    }
    let mut imm = ((raw >> 5) & 0xFF) as u8 as i8 as i64;
    if shift {
        imm <<= 8;
    }
    Some(Instr {
        op: Opcode::SveDupImm,
        rd: (raw & 0x1F) as u8,
        rn: 0,
        rm: 0xFF,
        imm: imm as u64,
        sf: false,
        cond: 0xFF,
        size,
    })
}

fn decode_indexed(raw: u32) -> Option<Instr> {
    if (raw & 0xFF20_FC00) != 0x0520_2000 {
        return None;
    }
    let tsz = (raw >> 16) & 0x1F;
    if tsz == 0 {
        return None;
    }
    let lsb = tsz.trailing_zeros();
    let size = 1u8 << lsb;
    let imm = (((raw >> 22) & 0x3) << 5) | tsz;
    Some(Instr {
        op: Opcode::SveDupElem,
        rd: (raw & 0x1F) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm: 0xFF,
        imm: (imm >> (lsb + 1)) as u64,
        sf: false,
        cond: 0xFF,
        size,
    })
}
