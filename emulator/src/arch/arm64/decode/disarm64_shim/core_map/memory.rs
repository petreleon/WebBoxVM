use crate::arch::arm64::opcodes::Opcode;

pub(super) fn map(raw: u32, m: disarm64::decoder::Mnemonic) -> Option<Opcode> {
    use disarm64::decoder::Mnemonic as M;
    Some(match m {
        M::r#prfm | M::r#prfum => Opcode::Prfm,
        M::r#ldr if matches!(raw & 0xFFC0_E000, 0x8580_0000 | 0x8580_4000) => Opcode::SveLdr,
        M::r#ldr if (raw & 0x3B00_0000) == 0x1800_0000 => Opcode::LdrLit,
        M::r#ldr | M::r#ldur if ((raw >> 26) & 1) != 0 => Opcode::SimdLdr,
        M::r#ldr | M::r#ldur | M::r#ldrb | M::r#ldurb | M::r#ldrh | M::r#ldurh => Opcode::Ldr,
        M::r#ldapur if is_rcpc3_simd_load(raw) => Opcode::SimdLdr,
        M::r#ldapur | M::r#ldapurb | M::r#ldapurh => Opcode::Ldapur,
        M::r#ldapursb | M::r#ldapursh | M::r#ldapursw => Opcode::Ldapurs,
        M::r#ldraa | M::r#ldrab => {
            if (raw & 0xFFA0_0400) == 0xF820_0400 {
                Opcode::Ldraa
            } else {
                Opcode::Ldrab
            }
        }
        M::r#ld1rd if (raw & 0xFFC0_E000) == 0x85C0_E000 => Opcode::SveLd1rd,
        M::r#ld1rw if (raw & 0xFFC0_E000) == 0x8540_C000 => Opcode::SveLd1rw,
        M::r#ld1rqd if (raw & 0xFFF0_E000) == 0xA580_2000 => Opcode::SveLd1rqd,
        M::r#ld1rqw if (raw & 0xFFF0_E000) == 0xA500_2000 => Opcode::SveLd1rqw,
        M::r#ld1d if (raw & 0xFFE0_E000) == 0xC5E0_C000 || (raw & 0xFFF0_E000) == 0xA5E0_A000 => {
            Opcode::SveLd1d
        }
        M::r#ld1w if sve_ld1w(raw) => Opcode::SveLd1w,
        M::r#st1d if (raw & 0xFFF0_E000) == 0xE5E0_E000 => Opcode::SveSt1d,
        M::r#st1w if (raw & 0xFF90_E000) == 0xE500_E000 => Opcode::SveSt1w,
        M::r#ldrsw if (raw & 0x3B00_0000) == 0x1800_0000 => Opcode::LdrLit,
        M::r#ldrsb | M::r#ldursb | M::r#ldrsh | M::r#ldursh | M::r#ldrsw | M::r#ldursw => {
            Opcode::LdrSign
        }
        M::r#str if matches!(raw & 0xFFC0_E000, 0xE580_0000 | 0xE580_4000) => Opcode::SveStr,
        M::r#str | M::r#stur if ((raw >> 26) & 1) != 0 => Opcode::SimdStr,
        M::r#stlur if is_rcpc3_simd_store(raw) => Opcode::SimdStr,
        M::r#stlur | M::r#stlurb | M::r#stlurh => Opcode::Stlur,
        M::r#str | M::r#stur | M::r#strb | M::r#sturb | M::r#strh | M::r#sturh => Opcode::Str,
        M::r#ldp if ((raw >> 26) & 1) != 0 => Opcode::SimdLdp,
        M::r#ldiapp => Opcode::Ldp,
        M::r#ldp => Opcode::Ldp,
        M::r#ldpsw => Opcode::Ldpsw,
        M::r#stp if ((raw >> 26) & 1) != 0 => Opcode::SimdStp,
        M::r#stilp => Opcode::Stp,
        M::r#stp => Opcode::Stp,
        _ => return None,
    })
}

fn sve_ld1w(raw: u32) -> bool {
    (raw & 0xFF90_E000) == 0xA500_A000 || matches!(raw & 0xFFA0_E000, 0x8520_4000 | 0xC520_C000)
}

fn is_rcpc3_simd_load(raw: u32) -> bool {
    (raw & 0x3F60_0C00) == 0x1D40_0800
}

fn is_rcpc3_simd_store(raw: u32) -> bool {
    (raw & 0x3F60_0C00) == 0x1D00_0800
}
