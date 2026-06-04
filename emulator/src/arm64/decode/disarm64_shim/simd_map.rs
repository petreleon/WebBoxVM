use super::*;

pub(super) fn map(raw: u32, m: disarm64::decoder::Mnemonic) -> Option<Opcode> {
    use disarm64::decoder::Mnemonic as M;
    Some(match m {
        M::r#aese => Opcode::SimdAese,
        M::r#aesd => Opcode::SimdAesd,
        M::r#aesmc => Opcode::SimdAesmc,
        M::r#aesimc => Opcode::SimdAesimc,
        M::r#pmull | M::r#pmull2 => Opcode::SimdPmull,
        M::r#sha1h => Opcode::SimdSha1h,
        M::r#sha256su0 => Opcode::SimdSha256Su0,
        M::r#sha512su0 => Opcode::SimdSha512Su0,
        M::r#sm4e => Opcode::SimdSm4e,
        M::r#sm3partw1 => Opcode::SimdSm3Partw1,
        M::r#eor3 => Opcode::SimdEor3,
        M::r#bcax => Opcode::SimdBcax,
        M::r#rax1 => Opcode::SimdRax1,
        M::r#xar => Opcode::SimdXar,
        M::r#sel if (raw & 0xFF20_C000) == 0x0520_C000 => Opcode::SveSel,
        M::r#addp => Opcode::SimdAddp,
        M::r#addhn if (raw & 0xFF20_FC00) == 0x0E20_4000 => Opcode::SimdAddhn,
        M::r#addhn2 if (raw & 0xFF20_FC00) == 0x4E20_4000 => Opcode::SimdAddhn2,
        M::r#raddhn if (raw & 0xFF20_FC00) == 0x2E20_4000 => Opcode::SimdRaddhn,
        M::r#raddhn2 if (raw & 0xFF20_FC00) == 0x6E20_4000 => Opcode::SimdRaddhn2,
        M::r#subhn if (raw & 0xFF20_FC00) == 0x0E20_6000 => Opcode::SimdSubhn,
        M::r#subhn2 if (raw & 0xFF20_FC00) == 0x4E20_6000 => Opcode::SimdSubhn2,
        M::r#rsubhn if (raw & 0xFF20_FC00) == 0x2E20_6000 => Opcode::SimdRsubhn,
        M::r#rsubhn2 if (raw & 0xFF20_FC00) == 0x6E20_6000 => Opcode::SimdRsubhn2,
        M::r#addv => Opcode::SimdAddv,
        M::r#smaxv if simd_across_minmax(raw, 0x0E30_A800) => Opcode::SimdSmaxv,
        M::r#sminv if simd_across_minmax(raw, 0x0E31_A800) => Opcode::SimdSminv,
        M::r#umaxv if simd_across_minmax(raw, 0x2E30_A800) => Opcode::SimdUmaxv,
        M::r#uminv if simd_across_minmax(raw, 0x2E31_A800) => Opcode::SimdUminv,
        M::r#cmeq
            if ((raw & 0xFF20_FC00) == 0x5E20_9800 && ((raw >> 22) & 0x3) == 0x3)
                || (raw & 0xBF3F_FC00) == 0x0E20_9800 =>
        {
            Opcode::SimdCmeqZero
        }
        M::r#cmge if (raw & 0xFF3F_FC00) == 0x7E20_8800 || (raw & 0xBF3F_FC00) == 0x2E20_8800 => {
            Opcode::SimdCmgeZero
        }
        M::r#cmgt if (raw & 0xFF3F_FC00) == 0x5E20_8800 || (raw & 0xBF3F_FC00) == 0x0E20_8800 => {
            Opcode::SimdCmgtZero
        }
        M::r#cmle if (raw & 0xFF3F_FC00) == 0x7E20_9800 || (raw & 0xBF3F_FC00) == 0x2E20_9800 => {
            Opcode::SimdCmleZero
        }
        M::r#cmlt if (raw & 0xFF3F_FC00) == 0x5E20_A800 || (raw & 0xBF3F_FC00) == 0x0E20_A800 => {
            Opcode::SimdCmltZero
        }
        M::r#cmeq if (raw & 0xBF20_FC00) == 0x2E20_8C00 || (raw & 0xFFE0_FC00) == 0x7EE0_8C00 => {
            Opcode::SimdCmeqReg
        }
        M::r#cmgt if (raw & 0xBF20_FC00) == 0x0E20_3400 || (raw & 0xFFE0_FC00) == 0x5EE0_3400 => {
            Opcode::SimdCmgtReg
        }
        M::r#cmge if (raw & 0xBF20_FC00) == 0x0E20_3C00 || (raw & 0xFFE0_FC00) == 0x5EE0_3C00 => {
            Opcode::SimdCmgeReg
        }
        M::r#cmhi if (raw & 0xBF20_FC00) == 0x2E20_3400 || (raw & 0xFFE0_FC00) == 0x7EE0_3400 => {
            Opcode::SimdCmhiReg
        }
        M::r#cmhs if (raw & 0xBF20_FC00) == 0x2E20_3C00 || (raw & 0xFFE0_FC00) == 0x7EE0_3C00 => {
            Opcode::SimdCmhsReg
        }
        M::r#cmphs if (raw & 0xFF20_2010) == 0x2420_0000 => Opcode::SveCmpHsImm,
        M::r#cmphs if (raw & 0xFF20_E010) == 0x2400_0000 => Opcode::SveCmpHs,
        M::r#abs if (raw & 0xBF3F_FC00) == 0x0E20_B800 || (raw & 0xFFFF_FC00) == 0x5EE0_B800 => {
            Opcode::SimdAbs
        }
        M::r#neg if (raw & 0xFF3F_FC00) == 0x7E20_B800 => Opcode::SimdNeg,
        M::r#neg if (raw & 0xBF3F_FC00) == 0x2E20_B800 => Opcode::SimdNeg,
        M::r#ext => Opcode::SimdExt,
        M::r#cnt => Opcode::SimdCnt,
        M::r#cmtst => Opcode::SimdCmtst,
        M::r#smax if simd_minmax(raw, 0x0E20_6400) => Opcode::SimdSmaxVec,
        M::r#smin if simd_minmax(raw, 0x0E20_6C00) => Opcode::SimdSminVec,
        M::r#umax if simd_minmax(raw, 0x2E20_6400) => Opcode::SimdUmaxVec,
        M::r#umin if simd_minmax(raw, 0x2E20_6C00) => Opcode::SimdUminVec,
        M::r#shl => Opcode::SimdShlImm,
        M::r#sli => Opcode::SimdSli,
        M::r#sri => Opcode::SimdSri,
        M::r#shrn => Opcode::SimdShrn,
        M::r#shrn2 => Opcode::SimdShrn2,
        M::r#rshrn => Opcode::SimdRshrn,
        M::r#rshrn2 => Opcode::SimdRshrn2,
        M::r#sshr => Opcode::SimdSshr,
        M::r#ushr => Opcode::SimdUshr,
        M::r#ushl => Opcode::SimdUshl,
        M::r#xtn => Opcode::SimdXtn,
        M::r#xtn2 => Opcode::SimdXtn2,
        M::r#rev64 if (raw & 0xBF3F_FC00) == 0x0E20_0800 => Opcode::SimdRev64,
        M::r#rev32 if (raw & 0xBF3F_FC00) == 0x2E20_0800 => Opcode::SimdRev32,
        M::r#uzp1 if (raw & 0xBF20_FC00) == 0x0E00_1800 => Opcode::SimdUzp1,
        M::r#uzp2 if (raw & 0xBF20_FC00) == 0x0E00_5800 => Opcode::SimdUzp2,
        M::r#trn1 if (raw & 0xBF20_FC00) == 0x0E00_2800 => Opcode::SimdTrn1,
        M::r#trn2 if (raw & 0xBF20_FC00) == 0x0E00_6800 => Opcode::SimdTrn2,
        M::r#zip1 if (raw & 0xBF20_FC00) == 0x0E00_3800 => Opcode::SimdZip1,
        M::r#zip2 if (raw & 0xBF20_FC00) == 0x0E00_7800 => Opcode::SimdZip2,
        M::r#tbl if (raw & 0xBFE0_9C00) == 0x0E00_0000 => Opcode::SimdTbl,
        M::r#not => Opcode::SimdNot,
        M::r#movi => Opcode::SimdMovi,
        M::r#mvni if (raw & 0xFFFF_FC00) == 0x6F00_0400 => Opcode::SimdMovi,
        M::r#mvni => Opcode::SimdMvni,
        M::r#ushll => Opcode::SimdUshll,
        M::r#sshll => Opcode::SimdSshll,
        M::r#shll | M::r#shll2 => Opcode::SimdShll,
        M::r#saddl | M::r#saddl2 => Opcode::SimdSaddl,
        M::r#usubl | M::r#usubl2 => Opcode::SimdUsubl,
        M::r#ssubw | M::r#ssubw2 => Opcode::SimdSsubw,
        M::r#umlal | M::r#umlal2 => Opcode::SimdUmlal,
        M::r#uqsub if (raw & 0xFF20_FC00) == 0x7E20_2C00 => Opcode::SimdUqsub,
        M::r#smaxp if simd_minmax(raw, 0x0E20_A400) => Opcode::SimdSmaxp,
        M::r#sminp if simd_minmax(raw, 0x0E20_AC00) => Opcode::SimdSminp,
        M::r#umaxp if simd_minmax(raw, 0x2E20_A400) => Opcode::SimdUmaxp,
        M::r#uminp if simd_minmax(raw, 0x2E20_AC00) => Opcode::SimdUminp,
        _ => return None,
    })
}

fn simd_minmax(raw: u32, base: u32) -> bool {
    (raw & 0xBF20_FC00) == base && ((raw >> 22) & 0x3) != 0x3
}

fn simd_across_minmax(raw: u32, base: u32) -> bool {
    let q = ((raw >> 30) & 1) != 0;
    let element_size = 1u64 << ((raw >> 22) & 0x3);
    (raw & 0xBF3F_FC00) == base && element_size < 8 && (element_size != 4 || q)
}
