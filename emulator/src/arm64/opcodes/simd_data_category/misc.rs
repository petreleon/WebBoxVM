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
            | Opcode::SimdSha1C
            | Opcode::SimdSha1M
            | Opcode::SimdSha1P
            | Opcode::SimdSha1Su0
            | Opcode::SimdSha1Su1
            | Opcode::SimdSha256H
            | Opcode::SimdSha256H2
            | Opcode::SimdSha256Su1
            | Opcode::SimdSm4e
            | Opcode::SimdSm4EKey
            | Opcode::SimdSm3Partw1
            | Opcode::SimdSm3Partw2
            | Opcode::SimdSm3Ss1
            | Opcode::SimdSm3Tt1A
            | Opcode::SimdSm3Tt1B
            | Opcode::SimdSm3Tt2A
            | Opcode::SimdSm3Tt2B
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
            | Opcode::SimdUqadd
            | Opcode::SimdAbs
            | Opcode::SimdNeg
            | Opcode::SimdCnt
            | Opcode::SimdCls
            | Opcode::SimdClz
            | Opcode::SimdCmtst
    )
}
