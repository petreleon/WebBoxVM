use super::*;

pub(in crate::arm64::decode) fn decode_movz(raw: u32) -> Option<Instr> {
    let sf = ((raw >> 31) & 1) != 0;
    if ((raw >> 29) & 3) != 2 {
        return None;
    }
    let hw = ((raw >> 21) & 3) as u64;
    if hw > (if sf { 3 } else { 1 }) {
        return None;
    }
    let imm16 = ((raw >> 5) & 0xFFFF) as u64;
    let rd = (raw & 0x1F) as u8;
    Some(Instr {
        size: 0,
        op: Opcode::Movz,
        rd,
        rn: 0,
        rm: 0,
        imm: imm16 << (hw * 16),
        sf,
        cond: 0,
    })
}

pub(in crate::arm64::decode) fn decode_movk(raw: u32) -> Option<Instr> {
    let sf = ((raw >> 31) & 1) != 0;
    if ((raw >> 29) & 3) != 3 {
        return None;
    }
    let hw = ((raw >> 21) & 3) as u8;
    if hw > (if sf { 3 } else { 1 }) {
        return None;
    }
    let imm16 = ((raw >> 5) & 0xFFFF) as u64;
    let rd = (raw & 0x1F) as u8;
    Some(Instr {
        size: 0,
        op: Opcode::Movk,
        rd,
        rn: 0,
        rm: 0,
        imm: imm16 << (hw as u64 * 16),
        sf,
        cond: hw,
    })
}

pub(in crate::arm64::decode) fn decode_movn(raw: u32) -> Option<Instr> {
    let sf = ((raw >> 31) & 1) != 0;
    if ((raw >> 29) & 3) != 0 {
        return None;
    }
    let hw = ((raw >> 21) & 3) as u64;
    if hw > (if sf { 3 } else { 1 }) {
        return None;
    }
    let imm16 = ((raw >> 5) & 0xFFFF) as u64;
    let rd = (raw & 0x1F) as u8;
    let val = !(imm16 << (hw * 16));
    Some(Instr {
        size: 0,
        op: Opcode::Movn,
        rd,
        rn: 0,
        rm: 0,
        imm: if sf { val } else { val & 0xFFFF_FFFF },
        sf,
        cond: 0,
    })
}
