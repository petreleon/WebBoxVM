use super::*;

pub(super) fn decode(raw: u32) -> DecodeStep {
    if let Some(instr) = decode_immediate(raw) {
        return DecodeStep::Hit(instr);
    }
    if let Some(instr) = decode_gather(raw) {
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
