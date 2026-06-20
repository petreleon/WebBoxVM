use super::*;

pub(in crate::runtime) fn is_sve_opcode(op: Opcode) -> bool {
    op.is_sve()
}

pub(in crate::runtime) fn is_simd_memory_or_crypto_opcode(op: Opcode) -> bool {
    op.is_simd_memory_or_crypto()
}
