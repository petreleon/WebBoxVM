use super::*;

pub(in crate::runtime) fn is_simd_integer_opcode(op: Opcode) -> bool {
    op.is_simd_data()
}
