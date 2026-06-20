use super::*;

pub(super) fn decode(raw: u32) -> DecodeStep {
    if let Some(step) = decode_predicate(raw) {
        return step;
    }
    decode_vector(raw)
}

fn decode_vector(raw: u32) -> DecodeStep {
    let op = match raw & 0xFF3F_FC00 {
        0x0530_3800 => Opcode::SveSunpklo,
        0x0531_3800 => Opcode::SveSunpkhi,
        0x0532_3800 => Opcode::SveUunpklo,
        0x0533_3800 => Opcode::SveUunpkhi,
        _ => return DecodeStep::Miss,
    };
    let size_bits = ((raw >> 22) & 0x3) as u8;
    if size_bits == 0 {
        return DecodeStep::Reject;
    }
    DecodeStep::Hit(Instr {
        op,
        rd: (raw & 0x1F) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm: 0,
        imm: 0,
        sf: true,
        cond: 0xFF,
        size: 1u8 << size_bits,
    })
}

fn decode_predicate(raw: u32) -> Option<DecodeStep> {
    let op = match raw & 0xFFFF_FC10 {
        0x0530_4000 => Opcode::SvePunpklo,
        0x0531_4000 => Opcode::SvePunpkhi,
        _ => return None,
    };
    Some(DecodeStep::Hit(Instr {
        op,
        rd: (raw & 0xF) as u8,
        rn: ((raw >> 5) & 0xF) as u8,
        rm: 0,
        imm: 0,
        sf: true,
        cond: 0xFF,
        size: 2,
    }))
}
