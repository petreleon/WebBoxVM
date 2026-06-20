use super::*;

pub(super) fn decode(raw: u32) -> DecodeStep {
    if (raw & 0xFF20_0C00) == 0x1900_0400 {
        return decode_cpyf_or_set(raw);
    }
    if (raw & 0xFF20_0C00) == 0x1D00_0400 {
        return decode_cpy(raw);
    }
    DecodeStep::Miss
}

fn decode_cpyf_or_set(raw: u32) -> DecodeStep {
    let stage = ((raw >> 22) & 0x3) as u8;
    if stage < 3 {
        return DecodeStep::Hit(decode_common(raw, cpyf_opcode(stage)));
    }

    match (raw >> 12) & 0xF {
        0x0 => DecodeStep::Hit(decode_common(raw, Opcode::MopsSetP)),
        0x4 => DecodeStep::Hit(decode_common(raw, Opcode::MopsSetM)),
        0x8 => DecodeStep::Hit(decode_common(raw, Opcode::MopsSetE)),
        _ => DecodeStep::Miss,
    }
}

fn decode_cpy(raw: u32) -> DecodeStep {
    match (raw >> 22) & 0x3 {
        0 => DecodeStep::Hit(decode_common(raw, Opcode::MopsCpyP)),
        1 => DecodeStep::Hit(decode_common(raw, Opcode::MopsCpyM)),
        2 => DecodeStep::Hit(decode_common(raw, Opcode::MopsCpyE)),
        _ => DecodeStep::Miss,
    }
}

fn decode_common(raw: u32, op: Opcode) -> Instr {
    Instr {
        op,
        rd: (raw & 0x1F) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm: ((raw >> 16) & 0x1F) as u8,
        imm: ((raw >> 12) & 0xF) as u64,
        sf: true,
        cond: 0,
        size: 0,
    }
}

fn cpyf_opcode(stage: u8) -> Opcode {
    match stage {
        0 => Opcode::MopsCpyFp,
        1 => Opcode::MopsCpyFm,
        2 => Opcode::MopsCpyFe,
        _ => unreachable!(),
    }
}
