use super::Opcode;

pub(super) fn is_opcode(op: Opcode) -> bool {
    matches!(
        op,
        Opcode::FpAdd
            | Opcode::FpSub
            | Opcode::FpMul
            | Opcode::FpFnmul
            | Opcode::FpDiv
            | Opcode::FpMaxnm
            | Opcode::FpMinnm
            | Opcode::FpNeg
            | Opcode::FpAbs
            | Opcode::FpSqrt
            | Opcode::FpFcvt
            | Opcode::FpFrintm
            | Opcode::FpFrintn
            | Opcode::FpFrinta
            | Opcode::FpFrintx
            | Opcode::FpFrintz
            | Opcode::FpFrintp
            | Opcode::FpFrinti
            | Opcode::FpMovImm
            | Opcode::Fmadd
            | Opcode::Fmsub
            | Opcode::Fnmsub
            | Opcode::Scvtf
            | Opcode::Ucvtf
            | Opcode::Fcvtns
            | Opcode::Fcvtms
            | Opcode::Fcvtzs
            | Opcode::Fcvtzu
            | Opcode::Fcvtas
            | Opcode::Fcmp
            | Opcode::Fcmpe
            | Opcode::Fccmp
            | Opcode::Fccmpe
            | Opcode::Fcsel
    )
}
