use crate::arm64::Opcode;

pub(super) fn is_opcode(op: Opcode) -> bool {
    matches!(
        op,
        Opcode::SimdScvtf
            | Opcode::SimdUcvtf
            | Opcode::SimdFcvtzs
            | Opcode::SimdFcvtzu
            | Opcode::SimdFcvtas
            | Opcode::SimdFpAddVec
            | Opcode::SimdFpSubVec
            | Opcode::SimdFpMulVec
            | Opcode::SimdFpMulx
            | Opcode::SimdFpMulxElem
            | Opcode::SimdFpDivVec
            | Opcode::SimdFpAbd
            | Opcode::SimdFpNeg
            | Opcode::SimdFpAbsVec
            | Opcode::SimdFpFrintnVec
            | Opcode::SimdFpFrintaVec
            | Opcode::SimdFpSqrtVec
            | Opcode::SimdFpFacgeVec
            | Opcode::SimdFpFacgtVec
            | Opcode::SimdFpFcmgeVec
            | Opcode::SimdFpFcmgtVec
            | Opcode::SimdFpFcmeqZero
            | Opcode::SimdFpFcmleZero
            | Opcode::SimdFpFcmltZero
            | Opcode::SimdFpFmlaVec
            | Opcode::SimdFpFmlsVec
            | Opcode::SimdFpFmlaElem
            | Opcode::SimdFpFmlsElem
            | Opcode::SimdFpMulElem
            | Opcode::SimdFpFmaxVec
            | Opcode::SimdFpFminVec
            | Opcode::SimdFpFmaxnmVec
            | Opcode::SimdFpFminnmVec
            | Opcode::SimdFpFmaxp
            | Opcode::SimdFpFminp
            | Opcode::SimdFpFmaxnmp
            | Opcode::SimdFpFminnmp
            | Opcode::SimdFpAddp
            | Opcode::SimdFpFmaxv
            | Opcode::SimdFpFminv
            | Opcode::SimdFpFmaxnmv
            | Opcode::SimdFpFminnmv
    )
}
