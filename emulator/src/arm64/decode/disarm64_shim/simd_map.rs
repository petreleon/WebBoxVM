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
        M::r#subhn if (raw & 0xFF20_FC00) == 0x0E20_6000 => Opcode::SimdSubhn,
        M::r#addv => Opcode::SimdAddv,
        M::r#umaxv => Opcode::SimdUmaxv,
        M::r#cmeq
            if ((raw & 0xFF20_FC00) == 0x5E20_9800 && ((raw >> 22) & 0x3) == 0x3)
                || (raw & 0xBF3F_FC00) == 0x0E20_9800 =>
        {
            Opcode::SimdCmeqZero
        }
        M::r#cmge if (raw & 0xFF3F_FC00) == 0x7E20_8800 || (raw & 0xBF3F_FC00) == 0x2E20_8800 => {
            Opcode::SimdCmgeZero
        }
        M::r#cmeq if (raw & 0xBF20_FC00) == 0x2E20_8C00 => Opcode::SimdCmeqReg,
        M::r#cmhi if (raw & 0xBF20_FC00) == 0x2E20_3400 || (raw & 0xFFE0_FC00) == 0x7EE0_3400 => {
            Opcode::SimdCmhiReg
        }
        M::r#cmhs if (raw & 0xBF20_FC00) == 0x2E20_3C00 => Opcode::SimdCmhsReg,
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
        M::r#smax if (raw & 0xBF20_FC00) == 0x0E20_6400 => Opcode::SimdSmaxVec,
        M::r#umax if (raw & 0xBF20_FC00) == 0x2E20_6400 => Opcode::SimdUmaxVec,
        M::r#umin if (raw & 0xBF20_FC00) == 0x2E20_6C00 => Opcode::SimdUminVec,
        M::r#shl => Opcode::SimdShlImm,
        M::r#sli => Opcode::SimdSli,
        M::r#sri => Opcode::SimdSri,
        M::r#shrn => Opcode::SimdShrn,
        M::r#sshr => Opcode::SimdSshr,
        M::r#ushr => Opcode::SimdUshr,
        M::r#ushl => Opcode::SimdUshl,
        M::r#xtn => Opcode::SimdXtn,
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
        M::r#umaxp if (raw & 0xFF20_FC00) == 0x6E20_A400 => Opcode::SimdUmaxp,
        M::r#uminp => Opcode::SimdUminp,
        _ => return None,
    })
}
