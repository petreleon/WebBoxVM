use super::*;

pub(super) fn map(raw: u32, m: disarm64::decoder::Mnemonic) -> Option<Opcode> {
    use disarm64::decoder::Mnemonic as M;
    Some(match m {
        M::r#fadd if (raw & 0xBFA0_FC00) == 0x0E20_D400 => Opcode::SimdFpAddVec,
        M::r#fadd if (raw & 0xFF3F_E000) == 0x6500_8000 => Opcode::SveFpAdd,
        M::r#fadd => Opcode::FpAdd,
        M::r#fsub if (raw & 0xBFA0_FC00) == 0x0EA0_D400 => Opcode::SimdFpSubVec,
        M::r#fsub if (raw & 0xFF3F_E000) == 0x6501_8000 => Opcode::SveFpSub,
        M::r#fsubr if (raw & 0xFF3F_E000) == 0x6503_8000 => Opcode::SveFpSubr,
        M::r#fsub => Opcode::FpSub,
        M::r#fmul if (raw & 0xBFA0_FC00) == 0x2E20_DC00 => Opcode::SimdFpMulVec,
        M::r#fmul if (raw & 0xBF00_F400) == 0x0F00_9000 => Opcode::SimdFpMulElem,
        M::r#fmul if (raw & 0xFF20_FC00) == 0x6420_2000 => Opcode::SveFpMulIndex,
        M::r#fmul if (raw & 0xFF20_FC00) == 0x6500_0800 => Opcode::SveFpMul,
        M::r#fmul if (raw & 0xFF3F_E3C0) == 0x651A_8000 => Opcode::SveFpMulImm,
        M::r#fmul if (raw & 0xFF3F_E000) == 0x6502_8000 => Opcode::SveFpMul,
        M::r#fmul => Opcode::FpMul,
        M::r#fnmul => Opcode::FpFnmul,
        M::r#fdiv if (raw & 0xBFA0_FC00) == 0x2E20_FC00 => Opcode::SimdFpDivVec,
        M::r#fdiv => Opcode::FpDiv,
        M::r#fabd if (raw & 0xFF20_FC00) == 0x7E20_D400 || (raw & 0xBFA0_FC00) == 0x2EA0_D400 => {
            Opcode::SimdFpAbd
        }
        M::r#fmaxnm => Opcode::FpMaxnm,
        M::r#fminnm => Opcode::FpMinnm,
        M::r#fneg if (raw & 0xBFBF_FC00) == 0x2EA0_F800 => Opcode::SimdFpNeg,
        M::r#fneg => Opcode::FpNeg,
        M::r#fabs => Opcode::FpAbs,
        M::r#fsqrt => Opcode::FpSqrt,
        M::r#fcvt => Opcode::FpFcvt,
        M::r#frintm => Opcode::FpFrintm,
        M::r#frintn => Opcode::FpFrintn,
        M::r#frinta => Opcode::FpFrinta,
        M::r#frintx => Opcode::FpFrintx,
        M::r#frintz => Opcode::FpFrintz,
        M::r#frintp => Opcode::FpFrintp,
        M::r#frinti => Opcode::FpFrinti,
        M::r#fmadd => Opcode::Fmadd,
        M::r#fmsub => Opcode::Fmsub,
        M::r#fnmsub => Opcode::Fnmsub,
        M::r#fmla if (raw & 0xBFA0_FC00) == 0x0E20_CC00 => Opcode::SimdFpFmlaVec,
        M::r#fmla if (raw & 0xBF00_F400) == 0x0F00_1000 => Opcode::SimdFpFmlaElem,
        M::r#fmla if (raw & 0xFF20_E000) == 0x6520_0000 => Opcode::SveFpFmla,
        M::r#fmla if (raw & 0xFF20_FC00) == 0x6420_0000 => Opcode::SveFpFmlaIndex,
        M::r#fmls if (raw & 0xBFA0_FC00) == 0x0EA0_CC00 => Opcode::SimdFpFmlsVec,
        M::r#fmls if (raw & 0xBF00_F400) == 0x0F00_5000 => Opcode::SimdFpFmlsElem,
        M::r#fmls if (raw & 0xFF20_E000) == 0x6520_2000 => Opcode::SveFpFmls,
        M::r#fmls if (raw & 0xFF20_FC00) == 0x6420_0400 => Opcode::SveFpFmlsIndex,
        M::r#fmad if (raw & 0xFF20_E000) == 0x6520_8000 => Opcode::SveFpFmad,
        M::r#fmsb if (raw & 0xFF20_E000) == 0x6520_A000 => Opcode::SveFpFmsb,
        M::r#fcsel => Opcode::Fcsel,
        M::r#scvtf if (raw & 0xFFBF_FC00) == 0x5E21_D800 || (raw & 0xBFBF_FC00) == 0x0E21_D800 => {
            Opcode::SimdScvtf
        }
        M::r#scvtf => Opcode::Scvtf,
        M::r#ucvtf if (raw & 0xFFBF_FC00) == 0x7E21_D800 || (raw & 0xBFBF_FC00) == 0x2E21_D800 => {
            Opcode::SimdUcvtf
        }
        M::r#ucvtf => Opcode::Ucvtf,
        M::r#fcvtns => Opcode::Fcvtns,
        M::r#fcvtms => Opcode::Fcvtms,
        M::r#fcvtzs if (raw & 0xFFBF_FC00) == 0x5EA1_B800 || (raw & 0xBFBF_FC00) == 0x0EA1_B800 => {
            Opcode::SimdFcvtzs
        }
        M::r#fcvtzs => Opcode::Fcvtzs,
        M::r#fcvtzu if (raw & 0xFFBF_FC00) == 0x7EA1_B800 => Opcode::SimdFcvtzu,
        M::r#fcvtzu => Opcode::Fcvtzu,
        M::r#fcvtas => Opcode::Fcvtas,
        M::r#fcmp => Opcode::Fcmp,
        M::r#fcmpe => Opcode::Fcmpe,
        M::r#fccmp => Opcode::Fccmp,
        M::r#fccmpe => Opcode::Fccmpe,
        M::r#fmov if (raw & 0xFFBF_FC00) == 0x1E20_4000 => Opcode::SimdFmovReg64,
        M::r#fmov if (raw & 0xFFFF_FC00) == 0x9E67_0000 => Opcode::SimdFmovGprToD,
        M::r#fmov if (raw & 0xFFFF_FC00) == 0x9E66_0000 => Opcode::SimdFmovDToGpr,
        M::r#fmov if (raw & 0x7F3F_FC00) == 0x1E27_0000 => Opcode::SimdFmovGprToS,
        M::r#fmov if (raw & 0x7F3F_FC00) == 0x1E26_0000 => Opcode::SimdFmovSToGpr,
        M::r#fmov if (raw & 0xFFFF_FC00) == 0x9EAE_0000 => Opcode::SimdFmovLaneToGpr,
        M::r#fmov if (raw & 0xFFFF_FC00) == 0x9EAF_0000 => Opcode::SimdInsGprLane,
        M::r#fmov if (raw & 0xBFF8_FC00) == 0x0F00_F400 || (raw & 0xFFF8_FC00) == 0x6F00_F400 => {
            Opcode::SimdFmovImm
        }
        M::r#fmov if (raw & 0xFF20_1C00) == 0x1E20_1000 => Opcode::FpMovImm,
        M::r#umov => Opcode::SimdUmov,
        M::r#smov if simd_smov_is_valid(raw) => Opcode::SimdSmov,
        M::r#dup if (raw & 0xFF3F_FC00) == 0x0520_3800 => Opcode::SveDupGpr,
        M::r#dup if (raw & 0xBFE0_FC00) == 0x0E00_0400 || (raw & 0xFFE0_FC00) == 0x5E00_0400 => {
            Opcode::SimdDupElem
        }
        M::r#dup => Opcode::SimdDupByte,
        M::r#ins if (raw & 0xFFE0_8400) == 0x6E00_0400 => Opcode::SimdInsElem,
        M::r#ins if (raw & 0xFFE0_FC00) == 0x4E00_1C00 => Opcode::SimdInsGprLane,
        _ => return None,
    })
}
