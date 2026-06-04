use super::*;

pub(in crate::arm64::machine) fn is_simd_integer_opcode(op: Opcode) -> bool {
    op.is_simd_data()
}
