use super::*;

pub(super) fn map(raw: u32, m: disarm64::decoder::Mnemonic) -> Option<Opcode> {
    use disarm64::decoder::Mnemonic as M;
    Some(match m {
        M::r#ld1b if sve_ld1b(raw) => Opcode::SveLd1b,
        M::r#ld1h if sve_ld1h(raw) => Opcode::SveLd1h,
        M::r#ldnt1sh if (raw & 0xBFE0_E000) == 0x8480_8000 => Opcode::SveLdnt1sh,
        M::r#st1b if sve_st1b(raw) => Opcode::SveSt1b,
        M::r#st1h if sve_st1h(raw) => Opcode::SveSt1h,
        _ => return None,
    })
}

fn sve_ld1b(raw: u32) -> bool {
    (raw & 0xFF90_E000) == 0xA400_A000 || (raw & 0xFF80_E000) == 0xA400_4000
}

fn sve_ld1h(raw: u32) -> bool {
    matches!(raw & 0xFFF0_E000, 0xA4A0_A000 | 0xA4C0_A000 | 0xA4E0_A000)
        || matches!(raw & 0xFFE0_E000, 0xC4C0_C000 | 0xC4E0_C000)
}

fn sve_st1b(raw: u32) -> bool {
    (raw & 0xFF90_E000) == 0xE400_E000 || (raw & 0xFF80_E000) == 0xE400_4000
}

fn sve_st1h(raw: u32) -> bool {
    matches!(raw & 0xFFF0_E000, 0xE4A0_E000 | 0xE4C0_E000 | 0xE4E0_E000)
        || (raw & 0xFF80_E000) == 0xE480_4000
}
