use super::*;

pub(super) fn decode(raw: u32) -> DecodeStep {
    if let Some(instr) = decode_immediate(raw) {
        return DecodeStep::Hit(instr);
    }
    if let Some(instr) = decode_predicated(raw) {
        return DecodeStep::Hit(instr);
    }
    DecodeStep::Miss
}

fn decode_immediate(raw: u32) -> Option<Instr> {
    let op = match raw & 0xFF3F_C000 {
        0x2520_C000 => Opcode::SveAddImm,
        0x2521_C000 => Opcode::SveSubImm,
        _ => return None,
    };
    let size = 1u8 << (((raw >> 22) & 0x3) as u8);
    let shift = ((raw >> 13) & 1) != 0;
    if size == 1 && shift {
        return None;
    }
    let imm = ((raw >> 5) & 0xFF) << if shift { 8 } else { 0 };
    Some(Instr {
        op,
        rd: (raw & 0x1F) as u8,
        rn: (raw & 0x1F) as u8,
        rm: 0xFF,
        imm: imm as u64,
        sf: false,
        cond: 0xFF,
        size,
    })
}

fn decode_predicated(raw: u32) -> Option<Instr> {
    let op = match raw & 0xFF3F_E000 {
        0x0400_0000 => Opcode::SveAddPred,
        0x0401_0000 => Opcode::SveSubPred,
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
