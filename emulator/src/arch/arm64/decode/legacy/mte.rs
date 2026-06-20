use super::*;

pub(super) fn decode(raw: u32) -> DecodeStep {
    if (raw & 0xFFC0_C000) == 0x9180_0000 {
        return DecodeStep::Hit(decode_tag_addsub(raw, Opcode::MteAddg));
    }
    if (raw & 0xFFC0_C000) == 0xD180_0000 {
        return DecodeStep::Hit(decode_tag_addsub(raw, Opcode::MteSubg));
    }
    if (raw & 0xFFE0_FC00) == 0x9AC0_1000 {
        return DecodeStep::Hit(decode_gpr(raw, Opcode::MteIrg));
    }
    if (raw & 0xFFE0_FC00) == 0x9AC0_1400 {
        return DecodeStep::Hit(decode_gpr(raw, Opcode::MteGmi));
    }
    if (raw & 0xFFE0_0C00) == 0xD960_0000 {
        return DecodeStep::Hit(decode_mem(raw, Opcode::MteLdg, 0));
    }
    let stgp_mode = ((raw >> 23) & 0x3) as u8;
    if (raw & 0x7E40_0000) == 0x6800_0000 && stgp_mode != 0 && ((raw >> 22) & 1) == 0 {
        return DecodeStep::Hit(decode_stgp(raw));
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

fn decode_tag_addsub(raw: u32, op: Opcode) -> Instr {
    Instr {
        op,
        rd: (raw & 0x1F) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm: 0xFF,
        imm: ((raw >> 16) & 0x3F) as u64 * 16,
        sf: true,
        cond: ((raw >> 10) & 0xF) as u8,
        size: 8,
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

fn decode_stgp(raw: u32) -> Instr {
    let imm7 = ((raw >> 15) & 0x7F) as i64;
    let signed = if (imm7 & 0x40) != 0 {
        imm7 - 0x80
    } else {
        imm7
    };
    Instr {
        op: Opcode::MteStgp,
        rd: (raw & 0x1F) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm: ((raw >> 10) & 0x1F) as u8,
        imm: (signed * 16) as u64,
        sf: true,
        cond: ((raw >> 23) & 0x3) as u8,
        size: 8,
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
