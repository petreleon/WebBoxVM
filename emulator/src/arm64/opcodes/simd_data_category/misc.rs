use crate::arm64::Opcode;

pub(super) fn is_opcode(op: Opcode) -> bool {
    matches!(
        op,
        Opcode::SimdAese
            | Opcode::SimdAesd
            | Opcode::SimdAesmc
            | Opcode::SimdAesimc
            | Opcode::SimdPmull
            | Opcode::SimdSha1h
            | Opcode::SimdSha256Su0
            | Opcode::SimdSha512Su0
            | Opcode::SimdSha512H
            | Opcode::SimdSha512H2
            | Opcode::SimdSha512Su1
            | Opcode::SimdSm4e
            | Opcode::SimdSm3Partw1
            | Opcode::SimdEor3
            | Opcode::SimdBcax
            | Opcode::SimdRax1
            | Opcode::SimdXar
            | Opcode::SimdDupByte
            | Opcode::SimdDupElem
            | Opcode::SimdFmovReg64
            | Opcode::SimdFmovGprToD
            | Opcode::SimdFmovGprToS
            | Opcode::SimdFmovDToGpr
            | Opcode::SimdFmovSToGpr
            | Opcode::SimdFmovLaneToGpr
            | Opcode::SimdFmovImm
            | Opcode::SimdUmov
            | Opcode::SimdSmov
            | Opcode::SimdInsGprLane
            | Opcode::SimdCmeqZero
            | Opcode::SimdCmgtZero
            | Opcode::SimdCmgeZero
            | Opcode::SimdCmleZero
            | Opcode::SimdCmltZero
            | Opcode::SimdCmeqReg
            | Opcode::SimdCmgtReg
            | Opcode::SimdCmgeReg
            | Opcode::SimdCmhsReg
            | Opcode::SimdCmhiReg
            | Opcode::SimdUqsub
            | Opcode::SimdAbs
            | Opcode::SimdNeg
            | Opcode::SimdCnt
            | Opcode::SimdCmtst
    )
}
