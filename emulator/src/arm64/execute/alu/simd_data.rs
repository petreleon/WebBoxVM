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
        | Opcode::SimdCmgeZero
        | Opcode::SimdCmeqReg
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
        Opcode::SimdFcvtas => exec_simd_fp_convert(cpu, instr),
        Opcode::SimdFpAbsVec
        | Opcode::SimdFpFrintnVec
        | Opcode::SimdFpFrintaVec
        | Opcode::SimdFpSqrtVec => exec_simd_fp_unary_more(cpu, instr),
        Opcode::SimdFpFacgeVec
        | Opcode::SimdFpFacgtVec
        | Opcode::SimdFpFcmgeVec
        | Opcode::SimdFpFcmgtVec
        | Opcode::SimdFpFcmeqZero
        | Opcode::SimdFpFcmleZero
        | Opcode::SimdFpFcmltZero => exec_simd_fp_compare(cpu, instr),
        Opcode::SimdFpFmlaVec
        | Opcode::SimdFpFmlsVec
        | Opcode::SimdFpFmlaElem
        | Opcode::SimdFpFmlsElem
        | Opcode::SimdFpMulElem => exec_simd_fp_fused(cpu, instr),
        Opcode::SimdShrn
        | Opcode::SimdAddhn
        | Opcode::SimdSubhn
        | Opcode::SimdAddVec
        | Opcode::SimdSubVec
        | Opcode::SimdMulVec
        | Opcode::SimdMlaVec
        | Opcode::SimdXtn => exec_simd_integer(cpu, instr),
        Opcode::SimdAddp
        | Opcode::SimdAddv
        | Opcode::SimdUmaxv
        | Opcode::SimdSmaxVec
        | Opcode::SimdUmaxVec
        | Opcode::SimdUminVec
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
        Opcode::SimdUshll
        | Opcode::SimdSshll
        | Opcode::SimdShll
        | Opcode::SimdSaddl
        | Opcode::SimdUsubl
        | Opcode::SimdSsubw
        | Opcode::SimdUmlal => exec_simd_widen(cpu, instr),
        _ => unreachable!(),
    }
}
