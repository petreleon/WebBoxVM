use super::*;

mod memory;

pub(super) fn map(raw: u32, m: disarm64::decoder::Mnemonic) -> Option<Opcode> {
    if let Some(opcode) = memory::map(raw, m) {
        return Some(opcode);
    }

    use disarm64::decoder::Mnemonic as M;
    Some(match m {
        M::r#add
            if ((raw & 0xFF20_FC00) == 0x5E20_8400 && ((raw >> 22) & 0x3) == 0x3)
                || (raw & 0xBF20_FC00) == 0x0E20_8400 =>
        {
            Opcode::SimdAddVec
        }
        M::r#add if (raw & 0xFF20_FC00) == 0x0420_0000 => Opcode::SveAddVec,
        M::r#add if (raw & 0x1F80_0000) == 0x1100_0000 => Opcode::AddImm,
        M::r#add if (raw & 0x1F20_0000) == 0x0B20_0000 => Opcode::AddExt,
        M::r#add => Opcode::Add,
        M::r#adds if (raw & 0x1F20_0000) == 0x0B20_0000 => Opcode::AddsExt,
        M::r#adds if (raw & 0x1F80_0000) == 0x1100_0000 => Opcode::AddsImm,
        M::r#adds => Opcode::Adds,
        M::r#sub if (raw & 0xBF20_FC00) == 0x2E20_8400 => Opcode::SimdSubVec,
        M::r#sub if (raw & 0xFF20_FC00) == 0x0420_0400 => Opcode::SveSubVec,
        M::r#uqadd if (raw & 0xFF20_FC00) == 0x0420_1400 => Opcode::SveUqadd,
        M::r#sub if (raw & 0x1F80_0000) == 0x1100_0000 => Opcode::SubImm,
        M::r#sub if (raw & 0x1F20_0000) == 0x0B20_0000 => Opcode::SubExt,
        M::r#sub => Opcode::Sub,
        M::r#subs if (raw & 0x1F20_0000) == 0x0B20_0000 && (raw & 0x1F) == 31 => Opcode::Cmp,
        M::r#subs if (raw & 0x1F20_0000) == 0x0B20_0000 => Opcode::SubsExt,
        M::r#subs if (raw & 0x1F80_0000) == 0x1100_0000 && (raw & 0x1F) == 31 => Opcode::CmpImm,
        M::r#subs if (raw & 0x1F80_0000) == 0x1100_0000 => Opcode::SubsImm,
        M::r#subs if (raw & 0x1F) == 31 => Opcode::Cmp,
        M::r#subs => Opcode::Subs,
        M::r#adc => Opcode::Adc,
        M::r#adcs => Opcode::Adcs,
        M::r#sbc => Opcode::Sbc,
        M::r#sbcs => Opcode::Sbcs,
        M::r#movz => Opcode::Movz,
        M::r#movk => Opcode::Movk,
        M::r#movn => Opcode::Movn,
        M::r#and if (raw & 0xBFE0_FC00) == 0x0E20_1C00 => Opcode::SimdAnd,
        M::r#and if (raw & 0xFFFC_0000) == 0x0580_0000 => Opcode::SveAndImm,
        M::r#and if (raw & 0xFFF0_C210) == 0x2500_4000 || (raw & 0xFFF0_C210) == 0x2540_4000 => {
            Opcode::SvePredAnd
        }
        M::r#and if (raw & 0x1F80_0000) == 0x1200_0000 => Opcode::AndImm,
        M::r#and => Opcode::AndReg,
        M::r#ands if (raw & 0xFFF0_C210) == 0x2540_4000 => Opcode::SvePredAnd,
        M::r#ands if (raw & 0x1F80_0000) == 0x1200_0000 => Opcode::AndsImm,
        M::r#ands => Opcode::AndsReg,
        M::r#bic if (raw & 0xBFE0_FC00) == 0x0E60_1C00 => Opcode::SimdBic,
        M::r#bic if scalar_logical_register(raw) => Opcode::AndReg,
        M::r#orr
            if (raw & 0xBFE0_FC00) == 0x0EA0_1C00
                || (raw & 0xBFF8_9C00) == 0x0F00_1400
                || (raw & 0xBFF8_DC00) == 0x0F00_9400 =>
        {
            if (raw & 0xBFE0_FC00) == 0x0EA0_1C00 {
                Opcode::SimdOrr
            } else {
                Opcode::SimdOrrImm
            }
        }
        M::r#orr if (raw & 0xFFFC_0000) == 0x0500_0000 => Opcode::SveOrrImm,
        M::r#orr if (raw & 0xFFE0_FC00) == 0x0460_3000 => Opcode::SveOrrVec,
        M::r#orr if (raw & 0xFFF0_C210) == 0x2580_4000 || (raw & 0xFFF0_C210) == 0x25C0_4000 => {
            Opcode::SvePredOrr
        }
        M::r#orr if (raw & 0x1F80_0000) == 0x1200_0000 => Opcode::OrrImm,
        M::r#orn if (raw & 0xBFE0_FC00) == 0x0EE0_1C00 => Opcode::SimdOrn,
        M::r#orn if scalar_logical_register(raw) => Opcode::OrrReg,
        M::r#orrs if (raw & 0xFFF0_C210) == 0x25C0_4000 => Opcode::SvePredOrr,
        M::r#dupm if (raw & 0xFFFC_0000) == 0x05C0_0000 => Opcode::SveDupm,
        M::r#bsl if (raw & 0xBFE0_FC00) == 0x2E60_1C00 => Opcode::SimdBsl,
        M::r#bit if (raw & 0xBFE0_FC00) == 0x2EA0_1C00 => Opcode::SimdBit,
        M::r#bif if (raw & 0xBFE0_FC00) == 0x2EE0_1C00 => Opcode::SimdBif,
        M::r#orr => Opcode::OrrReg,
        M::r#eor if (raw & 0xFFFC_0000) == 0x0540_0000 => Opcode::SveEorImm,
        M::r#eor if (raw & 0xFFE0_FC00) == 0x04A0_3000 => Opcode::SveEorVec,
        M::r#eor if (raw & 0xFFE0_FC00) == 0x6E20_1C00 || (raw & 0xFFE0_FC00) == 0x2E20_1C00 => {
            Opcode::SimdEor
        }
        M::r#eor if (raw & 0x1F80_0000) == 0x1200_0000 => Opcode::EorImm,
        M::r#eor => Opcode::EorReg,
        M::r#eon if scalar_logical_register(raw) => Opcode::EorReg,
        M::r#bics if scalar_logical_register(raw) => Opcode::AndsReg,
        M::r#csel => Opcode::Csel,
        M::r#csinc => Opcode::Csinc,
        M::r#csinv => Opcode::Csinv,
        M::r#csneg => Opcode::Csneg,
        M::r#b => Opcode::B,
        M::r#b_ | M::r#bc_ => Opcode::BCond,
        M::r#bl => Opcode::Bl,
        M::r#br => Opcode::Br,
        M::r#blr => Opcode::Blr,
        M::r#ret => Opcode::Ret,
        M::r#cbz => Opcode::Cbz,
        M::r#cbnz => Opcode::Cbnz,
        M::r#tbz => Opcode::Tbz,
        M::r#tbnz => Opcode::Tbnz,
        M::r#adr => Opcode::Adr,
        M::r#adrp => Opcode::Adrp,
        M::r#cntb | M::r#cnth | M::r#cntw | M::r#cntd => Opcode::SveCnt,
        M::r#addvl => Opcode::SveAddvl,
        M::r#addsvl => Opcode::SveAddsvl,
        M::r#addpl => Opcode::SveAddpl,
        M::r#ptrue => Opcode::SvePtrue,
        M::r#ptest => Opcode::SvePtest,
        M::r#movprfx => Opcode::SveMovprfx,
        M::r#mul if (raw & 0xBF20_FC00) == 0x0E20_9C00 => Opcode::SimdMulVec,
        M::r#mla if (raw & 0xBF20_FC00) == 0x0E20_9400 => Opcode::SimdMlaVec,
        M::r#mls if (raw & 0xBF20_FC00) == 0x2E20_9400 => Opcode::SimdMlsVec,
        M::r#madd | M::r#mul => Opcode::Madd,
        M::r#smaddl | M::r#umaddl => Opcode::Madd,
        M::r#msub => Opcode::Msub,
        M::r#smsubl | M::r#umsubl => Opcode::Msub,
        M::r#smulh => Opcode::Smulh,
        M::r#umulh => Opcode::Umulh,
        M::r#udiv => Opcode::Udiv,
        M::r#sdiv => Opcode::Sdiv,
        M::r#lsl | M::r#lslv => Opcode::Lslv,
        M::r#lsr | M::r#lsrv => Opcode::Lsrv,
        M::r#asr | M::r#asrv => Opcode::Asrv,
        M::r#rorv => Opcode::Rorv,
        M::r#crc32b | M::r#crc32h | M::r#crc32w | M::r#crc32x => Opcode::Crc32,
        M::r#crc32cb | M::r#crc32ch | M::r#crc32cw | M::r#crc32cx => Opcode::Crc32c,
        M::r#sxtw => Opcode::Sxtw,
        M::r#ccmn => Opcode::Ccmn,
        M::r#ccmp => Opcode::Ccmp,
        M::r#sbfm if (raw & 0x7F80_0000) == 0x1300_0000 => Opcode::Sbfm,
        M::r#bfm if (raw & 0x7F80_0000) == 0x3300_0000 => Opcode::Bfm,
        M::r#ubfm if (raw & 0x7F80_0000) == 0x5300_0000 => Opcode::Ubfm,
        _ => return None,
    })
}
fn scalar_logical_register(raw: u32) -> bool {
    (raw & 0x1F00_0000) == 0x0A00_0000
}
