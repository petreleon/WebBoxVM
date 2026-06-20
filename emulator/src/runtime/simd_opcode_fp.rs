use super::*;

pub(in crate::runtime) fn is_fp_scalar_opcode(op: Opcode) -> bool {
    op.is_fp_scalar()
}
