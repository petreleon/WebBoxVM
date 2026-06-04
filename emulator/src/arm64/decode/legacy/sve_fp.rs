use super::*;

pub(super) fn decode(raw: u32) -> DecodeStep {
    if let Some(step) = super::sve_fp_dup_imm::decode(raw) {
        return step;
    }
    if let Some(step) = try_decode_indexed(raw) {
        return step;
    }
    if let Some(step) = try_decode_unpredicated(raw) {
        return step;
    }
    if let Some(step) = try_decode_immediate(raw) {
        return step;
    }
    match try_decode_fused(raw) {
        Some(step) => step,
        None => decode_binary(raw),
    }
}

fn try_decode_indexed(raw: u32) -> Option<DecodeStep> {
    let op = match raw & 0xFF20_FC00 {
        0x6420_0000 => Opcode::SveFpFmlaIndex,
        0x6420_0400 => Opcode::SveFpFmlsIndex,
        0x6420_2000 => Opcode::SveFpMulIndex,
        _ => return None,
    };
    let (size, index, rm) = indexed_size_index_rm(raw);
    Some(DecodeStep::Hit(Instr {
        op,
        rd: (raw & 0x1F) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm,
        imm: index as u64,
        sf: true,
        cond: 0xFF,
        size,
    }))
}

fn indexed_size_index_rm(raw: u32) -> (u8, u8, u8) {
    if ((raw >> 23) & 1) == 0 {
        let index = ((((raw >> 22) & 1) << 2) | ((raw >> 19) & 0x3)) as u8;
        (2, index, ((raw >> 16) & 0x7) as u8)
    } else if ((raw >> 22) & 1) == 0 {
        let index = ((((raw >> 20) & 1) << 1) | ((raw >> 19) & 1)) as u8;
        (4, index, ((raw >> 16) & 0x7) as u8)
    } else {
        (8, ((raw >> 20) & 1) as u8, ((raw >> 16) & 0xF) as u8)
    }
}

fn try_decode_unpredicated(raw: u32) -> Option<DecodeStep> {
    if (raw & 0xFF20_FC00) != 0x6500_0800 {
        return None;
    }
    let size = 1u8 << (((raw >> 22) & 0x3) as u8);
    if size == 1 {
        return Some(DecodeStep::Reject);
    }
    Some(DecodeStep::Hit(Instr {
        op: Opcode::SveFpMul,
        rd: (raw & 0x1F) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm: ((raw >> 16) & 0x1F) as u8,
        imm: 0,
        sf: true,
        cond: 0xFF,
        size,
    }))
}

fn try_decode_immediate(raw: u32) -> Option<DecodeStep> {
    let op = match raw & 0xFF3F_E3C0 {
        0x6518_8000 => Opcode::SveFpAddImm,
        0x651A_8000 => Opcode::SveFpMulImm,
        _ => return None,
    };
    let size = 1u8 << (((raw >> 22) & 0x3) as u8);
    if size == 1 {
        return Some(DecodeStep::Reject);
    }
    let rd = (raw & 0x1F) as u8;
    Some(DecodeStep::Hit(Instr {
        op,
        rd,
        rn: rd,
        rm: 0,
        imm: ((raw >> 5) & 1) as u64,
        sf: true,
        cond: ((raw >> 10) & 0x7) as u8,
        size,
    }))
}

fn try_decode_fused(raw: u32) -> Option<DecodeStep> {
    let op = match raw & 0xFF20_E000 {
        0x6520_0000 => Opcode::SveFpFmla,
        0x6520_2000 => Opcode::SveFpFmls,
        0x6520_8000 => Opcode::SveFpFmad,
        0x6520_A000 => Opcode::SveFpFmsb,
        _ => return None,
    };
    let size = 1u8 << (((raw >> 22) & 0x3) as u8);
    if size == 1 {
        return Some(DecodeStep::Reject);
    }
    let rd = (raw & 0x1F) as u8;
    let (rn, rm, imm) = match op {
        Opcode::SveFpFmla | Opcode::SveFpFmls => {
            (((raw >> 5) & 0x1F) as u8, ((raw >> 16) & 0x1F) as u8, 0)
        }
        Opcode::SveFpFmad | Opcode::SveFpFmsb => {
            (rd, ((raw >> 5) & 0x1F) as u8, (raw >> 16) & 0x1F)
        }
        _ => unreachable!(),
    };
    Some(DecodeStep::Hit(Instr {
        op,
        rd,
        rn,
        rm,
        imm: imm as u64,
        sf: true,
        cond: ((raw >> 10) & 0x7) as u8,
        size,
    }))
}

fn decode_binary(raw: u32) -> DecodeStep {
    let op = match raw & 0xFF3F_E000 {
        0x6500_8000 => Opcode::SveFpAdd,
        0x6501_8000 => Opcode::SveFpSub,
        0x6502_8000 => Opcode::SveFpMul,
        0x6503_8000 => Opcode::SveFpSubr,
        _ => return DecodeStep::Miss,
    };
    let size = 1u8 << (((raw >> 22) & 0x3) as u8);
    if size == 1 {
        return DecodeStep::Reject;
    }
    let rd = (raw & 0x1F) as u8;
    DecodeStep::Hit(Instr {
        op,
        rd,
        rn: rd,
        rm: ((raw >> 5) & 0x1F) as u8,
        imm: 0,
        sf: true,
        cond: ((raw >> 10) & 0x7) as u8,
        size,
    })
}
