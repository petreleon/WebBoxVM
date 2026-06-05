use super::*;

pub(super) fn decode(raw: u32) -> DecodeStep {
    if let Some(op) = vector_compare_op(raw) {
        return DecodeStep::Hit(decode_compare_vec(raw, op));
    }
    if let Some((op, imm)) = immediate_compare_op(raw) {
        return DecodeStep::Hit(decode_compare_imm(raw, op, imm));
    }
    if let Some(op) = while_predicate_op(raw) {
        return DecodeStep::Hit(decode_while_predicate(raw, op));
    }
    DecodeStep::Miss
}

fn vector_compare_op(raw: u32) -> Option<Opcode> {
    match raw & 0xFF20_E010 {
        0x2400_0000 => Some(Opcode::SveCmpHs),
        0x2400_0010 => Some(Opcode::SveCmpHi),
        0x2400_A000 => Some(Opcode::SveCmpEq),
        0x2400_A010 => Some(Opcode::SveCmpNe),
        _ => None,
    }
}

fn immediate_compare_op(raw: u32) -> Option<(Opcode, u64)> {
    match raw & 0xFF20_E010 {
        0x2500_8000 => Some((Opcode::SveCmpEqImm, simm5(raw >> 16) as u64)),
        0x2500_8010 => Some((Opcode::SveCmpNeImm, simm5(raw >> 16) as u64)),
        _ => match raw & 0xFF20_2010 {
            0x2420_0000 => Some((Opcode::SveCmpHsImm, ((raw >> 14) & 0x7F) as u64)),
            0x2420_0010 => Some((Opcode::SveCmpHiImm, ((raw >> 14) & 0x7F) as u64)),
            _ => None,
        },
    }
}

fn decode_compare_vec(raw: u32, op: Opcode) -> Instr {
    Instr {
        op,
        rd: (raw & 0xF) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm: ((raw >> 16) & 0x1F) as u8,
        imm: 0,
        sf: true,
        cond: ((raw >> 10) & 0x7) as u8,
        size: 1u8 << (((raw >> 22) & 0x3) as u8),
    }
}

fn decode_compare_imm(raw: u32, op: Opcode, imm: u64) -> Instr {
    Instr {
        op,
        rd: (raw & 0xF) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm: 0,
        imm,
        sf: true,
        cond: ((raw >> 10) & 0x7) as u8,
        size: 1u8 << (((raw >> 22) & 0x3) as u8),
    }
}

fn simm5(value: u32) -> i64 {
    (((value & 0x1F) as i8) << 3 >> 3) as i64
}

fn while_predicate_op(raw: u32) -> Option<Opcode> {
    match raw & 0xFF20_FC10 {
        0x2520_0000 | 0x2520_1000 => Some(Opcode::SveWhileGe),
        0x2520_0010 | 0x2520_1010 => Some(Opcode::SveWhileGt),
        0x2520_0400 | 0x2520_1400 => Some(Opcode::SveWhileLt),
        0x2520_0410 | 0x2520_1410 => Some(Opcode::SveWhileLe),
        0x2520_0800 | 0x2520_1800 => Some(Opcode::SveWhileHs),
        0x2520_0810 | 0x2520_1810 => Some(Opcode::SveWhileHi),
        0x2520_0C00 | 0x2520_1C00 => Some(Opcode::SveWhileLo),
        0x2520_0C10 | 0x2520_1C10 => Some(Opcode::SveWhileLs),
        _ => None,
    }
}

fn decode_while_predicate(raw: u32, op: Opcode) -> Instr {
    Instr {
        op,
        rd: (raw & 0xF) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm: ((raw >> 16) & 0x1F) as u8,
        imm: 0,
        sf: ((raw >> 12) & 1) != 0,
        cond: 0,
        size: 1u8 << (((raw >> 22) & 0x3) as u8),
    }
}
