use super::*;

pub(super) fn map(raw: u32, m: disarm64::decoder::Mnemonic) -> Option<Opcode> {
    use disarm64::decoder::Mnemonic as M;
    Some(match m {
        M::r#ld1b if sve_ld1b(raw) => Opcode::SveLd1b,
        M::r#st1b if sve_st1b(raw) => Opcode::SveSt1b,
        _ => return None,
    })
}

fn sve_ld1b(raw: u32) -> bool {
    (raw & 0xFF90_E000) == 0xA400_A000 || (raw & 0xFF80_E000) == 0xA400_4000
}

fn sve_st1b(raw: u32) -> bool {
    (raw & 0xFF90_E000) == 0xE400_E000 || (raw & 0xFF80_E000) == 0xE400_4000
}
