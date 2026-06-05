use super::opcodes;
use super::*;

pub(super) fn logical_opcode(op: Opcode) -> u8 {
    match op {
        Opcode::AndImm | Opcode::AndReg => opcodes::OP_I64_AND,
        Opcode::OrrImm | Opcode::OrrReg => opcodes::OP_I64_OR,
        Opcode::EorImm | Opcode::EorReg => opcodes::OP_I64_XOR,
        _ => unreachable!(),
    }
}

pub(super) fn reg_offset(reg: u8) -> u64 {
    JIT_STATE_X_OFFSET + reg as u64 * 8
}

pub(super) fn can_emit_shift(shift_type: u8, amount: u64, sf: bool) -> bool {
    let width = if sf { 64 } else { 32 };
    if amount >= width {
        return false;
    }
    amount == 0 || shift_type != 3 || sf
}

pub(super) fn can_emit_bitfield(instr: crate::arm64::Instr) -> bool {
    let size = bitfield_size(instr.sf);
    (instr.rm as u32) < size && (instr.imm as u32) < size
}

pub(super) fn bitfield_size(sf: bool) -> u32 {
    if sf { 64 } else { 32 }
}

pub(super) fn bitfield_mask(len: u32) -> u64 {
    if len >= 64 {
        u64::MAX
    } else {
        (1u64 << len) - 1
    }
}
