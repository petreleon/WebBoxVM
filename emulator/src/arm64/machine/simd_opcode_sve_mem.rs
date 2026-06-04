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
            | Opcode::SveOrrVec
            | Opcode::SveEorVec
            | Opcode::SveSel
            | Opcode::SveLdr
            | Opcode::SveStr
            | Opcode::SveLd1rd
            | Opcode::SveLd1rqd
            | Opcode::SveLd1d
            | Opcode::SveSt1d
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
