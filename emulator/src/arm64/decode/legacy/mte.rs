use super::*;

pub(super) fn decode(raw: u32) -> DecodeStep {
    if (raw & 0xFFE0_FC00) == 0x9AC0_1000 {
        return DecodeStep::Hit(decode_gpr(raw, Opcode::MteIrg));
    }
    if (raw & 0xFFE0_FC00) == 0x9AC0_1400 {
        return DecodeStep::Hit(decode_gpr(raw, Opcode::MteGmi));
    }
    if (raw & 0xFFE0_0C00) == 0xD960_0000 {
        return DecodeStep::Hit(decode_mem(raw, Opcode::MteLdg, 0));
    }

    let mode = ((raw >> 10) & 0x3) as u8;
    if mode == 0 {
        return DecodeStep::Miss;
    }

    match raw & 0xFFE0_0000 {
        0xD920_0000 => DecodeStep::Hit(decode_mem(raw, Opcode::MteStg, mode)),
        0xD960_0000 => DecodeStep::Hit(decode_mem(raw, Opcode::MteStzg, mode)),
        0xD9A0_0000 => DecodeStep::Hit(decode_mem(raw, Opcode::MteSt2g, mode)),
        0xD9E0_0000 => DecodeStep::Hit(decode_mem(raw, Opcode::MteStz2g, mode)),
        _ => DecodeStep::Miss,
    }
}

fn decode_gpr(raw: u32, op: Opcode) -> Instr {
    Instr {
        op,
        rd: (raw & 0x1F) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm: ((raw >> 16) & 0x1F) as u8,
        imm: 0,
        sf: true,
        cond: 0,
        size: 0,
    }
}

fn decode_mem(raw: u32, op: Opcode, mode: u8) -> Instr {
    Instr {
        op,
        rd: (raw & 0x1F) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm: 0xFF,
        imm: signed_imm9_granule(raw),
        sf: true,
        cond: writeback_mode(mode),
        size: tag_store_size(op),
    }
}

fn signed_imm9_granule(raw: u32) -> u64 {
    let imm9 = ((raw >> 12) & 0x1FF) as i64;
    let signed = if (imm9 & 0x100) != 0 {
        imm9 | !0x1FF
    } else {
        imm9
    };
    (signed << 4) as u64
}

fn writeback_mode(mode: u8) -> u8 {
    match mode {
        1 => 1,
        3 => 3,
        _ => 0,
    }
}

fn tag_store_size(op: Opcode) -> u8 {
    match op {
        Opcode::MteSt2g | Opcode::MteStz2g => 32,
        Opcode::MteStg | Opcode::MteStzg => 16,
        _ => 0,
    }
}
