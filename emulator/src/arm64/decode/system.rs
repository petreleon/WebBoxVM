//! System instruction decoders: NOP, WFI, WFE, TLBI, barriers.

use super::{Instr, Opcode};

pub(super) fn decode_nop() -> Option<Instr> {
    Some(Instr { op: Opcode::Nop, rd: 0, rn: 0, rm: 0, imm: 0, sf: true, cond: 0, size: 0 })
}

pub(super) fn decode_wfi() -> Option<Instr> {
    Some(Instr { op: Opcode::Wfi, rd: 0, rn: 0, rm: 0, imm: 0, sf: true, cond: 0, size: 0 })
}

pub(super) fn decode_wfe() -> Option<Instr> {
    Some(Instr { op: Opcode::Wfe, rd: 0, rn: 0, rm: 0, imm: 0, sf: true, cond: 0, size: 0 })
}

pub(super) fn decode_tlbi(raw: u32) -> Option<Instr> {
    let op1 = ((raw >> 16) & 0x7) as u8;
    let crm = ((raw >> 8) & 0xF) as u8;
    let op2 = ((raw >> 5) & 0x7) as u8;
    let rt = (raw & 0x1F) as u8;
    let variant = ((op1 as u64) << 16) | ((crm as u64) << 8) | ((op2 as u64) << 4) | (rt as u64);
    Some(Instr { op: Opcode::Tlbi, rd: 0, rn: 0, rm: 0, imm: variant, sf: true, cond: 0, size: 0 })
}

pub(super) fn decode_barrier() -> Option<Instr> {
    Some(Instr { op: Opcode::NopBarrier, rd: 0, rn: 0, rm: 0, imm: 0, sf: true, cond: 0, size: 0 })
}
