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
