mod fp;
mod integer;
mod misc;

use super::Opcode;

pub(super) fn is_opcode(op: Opcode) -> bool {
    misc::is_opcode(op) || integer::is_opcode(op) || fp::is_opcode(op)
}
