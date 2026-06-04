use super::*;

pub(in crate::arm64::execute) fn exec_simd_data(cpu: &mut Armv8Cpu, instr: Instr) {
    match instr.op {
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
        | Opcode::SimdXar => exec_simd_crypto(cpu, instr),
        Opcode::SimdDupByte
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
        | Opcode::SimdInsGprLane => exec_simd_moves(cpu, instr),
        Opcode::SimdCmeqZero
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
        | Opcode::SimdCmtst => exec_simd_unary_compare(cpu, instr),
        Opcode::SimdScvtf
        | Opcode::SimdUcvtf
        | Opcode::SimdFcvtzs
        | Opcode::SimdFcvtzu
        | Opcode::SimdFpAddVec
        | Opcode::SimdFpSubVec
        | Opcode::SimdFpMulVec
        | Opcode::SimdFpDivVec
        | Opcode::SimdFpAbd
        | Opcode::SimdFpNeg => exec_simd_fp(cpu, instr),
        Opcode::SimdFpMulx => exec_simd_fp_mulx(cpu, instr),
        Opcode::SimdFpMulxElem => exec_simd_fp_mulx_elem(cpu, instr),
        Opcode::SimdFcvtas => exec_simd_fp_convert(cpu, instr),
        Opcode::SimdFpAbsVec
        | Opcode::SimdFpFrintnVec
        | Opcode::SimdFpFrintaVec
        | Opcode::SimdFpSqrtVec => exec_simd_fp_unary_more(cpu, instr),
        Opcode::SimdFpFacgeVec
        | Opcode::SimdFpFacgtVec
        | Opcode::SimdFpFcmeqVec
        | Opcode::SimdFpFcmgeVec
        | Opcode::SimdFpFcmgtVec
        | Opcode::SimdFpFcmeqZero
        | Opcode::SimdFpFcmgeZero
        | Opcode::SimdFpFcmgtZero
        | Opcode::SimdFpFcmleZero
        | Opcode::SimdFpFcmltZero => exec_simd_fp_compare(cpu, instr),
        Opcode::SimdFpFmlaVec
        | Opcode::SimdFpFmlsVec
        | Opcode::SimdFpFmlaElem
        | Opcode::SimdFpFmlsElem
        | Opcode::SimdFpMulElem => exec_simd_fp_fused(cpu, instr),
        Opcode::SimdFpFmaxVec
        | Opcode::SimdFpFminVec
        | Opcode::SimdFpFmaxnmVec
        | Opcode::SimdFpFminnmVec
        | Opcode::SimdFpFmaxp
        | Opcode::SimdFpFminp
        | Opcode::SimdFpFmaxnmp
        | Opcode::SimdFpFminnmp => exec_simd_fp_minmax(cpu, instr),
        Opcode::SimdFpAddp => exec_simd_fp_pairwise(cpu, instr),
        Opcode::SimdFpFmaxv
        | Opcode::SimdFpFminv
        | Opcode::SimdFpFmaxnmv
        | Opcode::SimdFpFminnmv => exec_simd_fp_reduce(cpu, instr),
        Opcode::SimdShrn
        | Opcode::SimdShrn2
        | Opcode::SimdRshrn
        | Opcode::SimdRshrn2
        | Opcode::SimdAddhn
        | Opcode::SimdAddhn2
        | Opcode::SimdRaddhn
        | Opcode::SimdRaddhn2
        | Opcode::SimdSubhn
        | Opcode::SimdSubhn2
        | Opcode::SimdRsubhn
        | Opcode::SimdRsubhn2
        | Opcode::SimdAddVec
        | Opcode::SimdSubVec
        | Opcode::SimdMulVec
        | Opcode::SimdMlaVec
        | Opcode::SimdXtn
        | Opcode::SimdXtn2 => exec_simd_integer(cpu, instr),
        Opcode::SimdAddp
        | Opcode::SimdAddv
        | Opcode::SimdSmaxv
        | Opcode::SimdSminv
        | Opcode::SimdUmaxv
        | Opcode::SimdUminv
        | Opcode::SimdSmaxVec
        | Opcode::SimdSminVec
        | Opcode::SimdUmaxVec
        | Opcode::SimdUminVec
        | Opcode::SimdSmaxp
        | Opcode::SimdSminp
        | Opcode::SimdUmaxp
        | Opcode::SimdUminp => exec_simd_reduce(cpu, instr),
        Opcode::SimdShlImm
        | Opcode::SimdSli
        | Opcode::SimdSri
        | Opcode::SimdSshr
        | Opcode::SimdUshr
        | Opcode::SimdUshl => exec_simd_shift(cpu, instr),
        Opcode::SimdExt
        | Opcode::SimdRev64
        | Opcode::SimdRev32
        | Opcode::SimdInsElem
        | Opcode::SimdUzp1
        | Opcode::SimdUzp2
        | Opcode::SimdTrn1
        | Opcode::SimdTrn2
        | Opcode::SimdZip1
        | Opcode::SimdZip2
        | Opcode::SimdTbl => exec_simd_permute(cpu, instr),
        Opcode::SimdNot
        | Opcode::SimdBsl
        | Opcode::SimdBit
        | Opcode::SimdBif
        | Opcode::SimdAnd
        | Opcode::SimdOrr
        | Opcode::SimdOrn
        | Opcode::SimdEor
        | Opcode::SimdBic
        | Opcode::SimdBicImm
        | Opcode::SimdMvni => exec_simd_logic(cpu, instr),
        op if is_simd_widen_opcode(op) => exec_simd_widen(cpu, instr),
        _ => unreachable!(),
    }
}
