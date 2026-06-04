use super::*;

pub(in crate::arm64::machine) fn is_fp_scalar_opcode(op: Opcode) -> bool {
    op.is_fp_scalar()
}
