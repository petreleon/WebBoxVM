use super::*;

pub(super) fn decode(raw: u32) -> DecodeStep {
    if (raw & 0xFF20_2010) == 0x2420_0000 {
        return DecodeStep::Hit(decode_cmphs_imm(raw));
    }
    if (raw & 0xFF20_E010) == 0x2400_0000 {
        return DecodeStep::Hit(decode_cmphs_vec(raw));
    }
    if (raw & 0xFF20_EC10) == 0x2520_0C00 {
        return DecodeStep::Hit(decode_whilelo(raw));
    }
    DecodeStep::Miss
}

fn decode_cmphs_vec(raw: u32) -> Instr {
    Instr {
        op: Opcode::SveCmpHs,
        rd: (raw & 0xF) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm: ((raw >> 16) & 0x1F) as u8,
        imm: 0,
        sf: true,
        cond: ((raw >> 10) & 0x7) as u8,
        size: 1u8 << (((raw >> 22) & 0x3) as u8),
    }
}

fn decode_cmphs_imm(raw: u32) -> Instr {
    Instr {
        op: Opcode::SveCmpHsImm,
        rd: (raw & 0xF) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm: 0,
        imm: ((raw >> 14) & 0x7F) as u64,
        sf: true,
        cond: ((raw >> 10) & 0x7) as u8,
        size: 1u8 << (((raw >> 22) & 0x3) as u8),
    }
}

fn decode_whilelo(raw: u32) -> Instr {
    Instr {
        op: Opcode::SveWhileLo,
        rd: (raw & 0xF) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm: ((raw >> 16) & 0x1F) as u8,
        imm: 0,
        sf: ((raw >> 12) & 1) != 0,
        cond: 0,
        size: 1u8 << (((raw >> 22) & 0x3) as u8),
    }
}
