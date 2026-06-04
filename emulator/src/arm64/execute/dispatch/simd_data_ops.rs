use crate::arm64::Opcode;

pub(super) fn is_simd_data_opcode(op: Opcode) -> bool {
    op.is_simd_data()
}
