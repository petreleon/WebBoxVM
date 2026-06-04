use super::*;

pub(in crate::arm64::machine) fn is_sve_opcode(op: Opcode) -> bool {
    matches!(
        op,
        Opcode::SvePtrue
            | Opcode::SvePtest
            | Opcode::SvePredAnd
            | Opcode::SvePredOrr
            | Opcode::SveMovprfx
            | Opcode::SveDupGpr
            | Opcode::SveAddVec
            | Opcode::SveSubVec
            | Opcode::SveAsrImm
            | Opcode::SveLsrImm
            | Opcode::SveLslImm
            | Opcode::SveOrrVec
            | Opcode::SveEorVec
            | Opcode::SveAndImm
            | Opcode::SveOrrImm
            | Opcode::SveEorImm
            | Opcode::SveDupm
            | Opcode::SveSel
            | Opcode::SveCmpHs
            | Opcode::SveCmpHsImm
            | Opcode::SveLdr
            | Opcode::SveStr
            | Opcode::SveLd1rd
            | Opcode::SveLd1rqd
            | Opcode::SveLd1d
            | Opcode::SveSt1d
            | Opcode::SveSt1b
            | Opcode::SveLd1b
            | Opcode::SveLd1rw
            | Opcode::SveLd1rqw
            | Opcode::SveLd1w
            | Opcode::SveSt1w
            | Opcode::SveFpAdd
            | Opcode::SveFpAddImm
            | Opcode::SveFpSub
            | Opcode::SveFpMul
            | Opcode::SveFpDiv
            | Opcode::SveFpSubr
            | Opcode::SveFpDivr
            | Opcode::SveFpMulImm
            | Opcode::SveFpDupImm
            | Opcode::SveFpAbs
            | Opcode::SveFpNeg
            | Opcode::SveFpFacge
            | Opcode::SveFpFacgt
            | Opcode::SveFpFcmeq
            | Opcode::SveFpFcmge
            | Opcode::SveFpFcmgt
            | Opcode::SveFpFcmne
            | Opcode::SveFpFcmle
            | Opcode::SveFpFcmlt
            | Opcode::SveFpFmla
            | Opcode::SveFpFmls
            | Opcode::SveFpFmad
            | Opcode::SveFpFmsb
            | Opcode::SveFpFmlaIndex
            | Opcode::SveFpFmlsIndex
            | Opcode::SveFpMulIndex
    )
}

pub(in crate::arm64::machine) fn is_simd_memory_or_crypto_opcode(op: Opcode) -> bool {
    matches!(
        op,
        Opcode::SimdLdp
            | Opcode::SimdStp
            | Opcode::SimdLdr
            | Opcode::SimdStr
            | Opcode::SimdMovi
            | Opcode::SimdLd1
            | Opcode::SimdLd1Multi
            | Opcode::SimdLd1Lane
            | Opcode::SimdLd1r
            | Opcode::SimdLd2
            | Opcode::SimdLd3
            | Opcode::SimdSt1Multi
            | Opcode::SimdSt1Lane
            | Opcode::SimdLd4
            | Opcode::SimdSt4Single
            | Opcode::SimdSt4
            | Opcode::SimdAese
            | Opcode::SimdAesd
            | Opcode::SimdAesmc
            | Opcode::SimdAesimc
            | Opcode::SimdPmull
            | Opcode::SimdSha1h
            | Opcode::SimdSha256Su0
            | Opcode::SimdSha512Su0
            | Opcode::SimdSm4e
            | Opcode::SimdSm3Partw1
            | Opcode::SimdEor3
            | Opcode::SimdBcax
            | Opcode::SimdRax1
            | Opcode::SimdXar
    )
}
