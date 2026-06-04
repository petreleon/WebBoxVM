use super::*;

pub(super) fn decode(raw: u32) -> DecodeStep {
    if let Some(instr) = decode_unpredicated_and(raw) {
        return DecodeStep::Hit(instr);
    }
    if let Some(instr) = decode_vector_predicated(raw) {
        return DecodeStep::Hit(instr);
    }
    if let Some(instr) = decode_predicate_logical(raw) {
        return DecodeStep::Hit(instr);
    }
    DecodeStep::Miss
}

fn decode_unpredicated_and(raw: u32) -> Option<Instr> {
    if (raw & 0xFFE0_FC00) != 0x0420_3000 {
        return None;
    }
    Some(Instr {
        op: Opcode::SveAndVec,
        rd: (raw & 0x1F) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm: ((raw >> 16) & 0x1F) as u8,
        imm: 0,
        sf: false,
        cond: 0,
        size: 8,
    })
}

fn decode_vector_predicated(raw: u32) -> Option<Instr> {
    let op = match raw & 0xFF3F_E000 {
        0x0418_0000 => Opcode::SveOrrPred,
        0x0419_0000 => Opcode::SveEorPred,
        0x041A_0000 => Opcode::SveAndPred,
        _ => return None,
    };
    Some(Instr {
        op,
        rd: (raw & 0x1F) as u8,
        rn: (raw & 0x1F) as u8,
        rm: ((raw >> 5) & 0x1F) as u8,
        imm: 0,
        sf: false,
        cond: ((raw >> 10) & 0x7) as u8,
        size: 1u8 << (((raw >> 22) & 0x3) as u8),
    })
}

fn decode_predicate_logical(raw: u32) -> Option<Instr> {
    let op = match (raw & 0xFFF0_C210) & !0x0040_0000 {
        0x2500_4000 => Opcode::SvePredAnd,
        0x2500_4010 => Opcode::SvePredBic,
        0x2500_4200 => Opcode::SvePredEor,
        0x2580_4000 => Opcode::SvePredOrr,
        0x2580_4010 => Opcode::SvePredOrn,
        0x2580_4200 => Opcode::SvePredNor,
        0x2580_4210 => Opcode::SvePredNand,
        _ => return None,
    };
    Some(Instr {
        op,
        rd: (raw & 0xF) as u8,
        rn: ((raw >> 5) & 0xF) as u8,
        rm: ((raw >> 16) & 0xF) as u8,
        imm: 0,
        sf: (raw & 0x0040_0000) != 0,
        cond: ((raw >> 10) & 0xF) as u8,
        size: 1,
    })
}
