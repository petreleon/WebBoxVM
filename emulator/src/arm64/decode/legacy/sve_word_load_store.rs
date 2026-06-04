use super::*;

pub(super) fn decode(raw: u32) -> DecodeStep {
    if let Some(instr) = decode_immediate(raw) {
        return DecodeStep::Hit(instr);
    }
    if let Some(instr) = decode_gather(raw) {
        return DecodeStep::Hit(instr);
    }
    if let Some(instr) = decode_halfword_load(raw) {
        return DecodeStep::Hit(instr);
    }
    DecodeStep::Miss
}

fn decode_immediate(raw: u32) -> Option<Instr> {
    let op = match raw & 0xFF90_E000 {
        0xA500_A000 => Opcode::SveLd1w,
        0xE500_E000 => Opcode::SveSt1w,
        _ => return None,
    };
    let signed_imm = (((((raw >> 16) & 0xF) as i32) << 28) >> 28) as i64;
    Some(Instr {
        op,
        rd: (raw & 0x1F) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm: 0xFF,
        imm: signed_imm as u64,
        sf: false,
        cond: ((raw >> 10) & 0x7) as u8,
        size: 1u8 << (((raw >> 21) & 0x3) as u8),
    })
}

fn decode_gather(raw: u32) -> Option<Instr> {
    let (size, signed) = match raw & 0xFFA0_E000 {
        0x8520_4000 => (4, (raw & 0x0040_0000) != 0),
        0xC520_C000 => (8, false),
        _ => return None,
    };
    Some(Instr {
        op: Opcode::SveLd1w,
        rd: (raw & 0x1F) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm: ((raw >> 16) & 0x1F) as u8,
        imm: 0,
        sf: signed,
        cond: ((raw >> 10) & 0x7) as u8,
        size,
    })
}

fn decode_halfword_load(raw: u32) -> Option<Instr> {
    if let Some(instr) = decode_ldnt1sh(raw) {
        return Some(instr);
    }
    if let Some(size) = ld1h_immediate_size(raw) {
        let signed_imm = (((((raw >> 16) & 0xF) as i32) << 28) >> 28) as i64;
        return Some(Instr {
            op: Opcode::SveLd1h,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: 0xFF,
            imm: signed_imm as u64,
            sf: false,
            cond: ((raw >> 10) & 0x7) as u8,
            size,
        });
    }
    let scaled = (raw & 0xFFE0_E000) == 0xC4E0_C000;
    if scaled || (raw & 0xFFE0_E000) == 0xC4C0_C000 {
        return Some(Instr {
            op: Opcode::SveLd1h,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: ((raw >> 16) & 0x1F) as u8,
            imm: 0,
            sf: scaled,
            cond: ((raw >> 10) & 0x7) as u8,
            size: 8,
        });
    }
    None
}

fn ld1h_immediate_size(raw: u32) -> Option<u8> {
    match raw & 0xFFF0_E000 {
        0xA4A0_A000 => Some(2),
        0xA4C0_A000 => Some(4),
        0xA4E0_A000 => Some(8),
        _ => None,
    }
}

fn decode_ldnt1sh(raw: u32) -> Option<Instr> {
    if (raw & 0xBFE0_E000) != 0x8480_8000 {
        return None;
    }
    Some(Instr {
        op: Opcode::SveLdnt1sh,
        rd: (raw & 0x1F) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm: ((raw >> 16) & 0x1F) as u8,
        imm: 0,
        sf: false,
        cond: ((raw >> 10) & 0x7) as u8,
        size: if (raw & 0x4000_0000) != 0 { 8 } else { 4 },
    })
}
