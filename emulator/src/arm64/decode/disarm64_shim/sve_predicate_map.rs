use super::*;

pub(super) fn map(raw: u32, m: disarm64::decoder::Mnemonic) -> Option<Opcode> {
    use disarm64::decoder::Mnemonic as M;
    match m {
        M::r#whilelt if matches!(raw & 0xFF20_FC10, 0x2520_0400 | 0x2520_1400) => {
            Some(Opcode::SveWhileLt)
        }
        M::r#whilele if matches!(raw & 0xFF20_FC10, 0x2520_0410 | 0x2520_1410) => {
            Some(Opcode::SveWhileLe)
        }
        M::r#whilelo if matches!(raw & 0xFF20_FC10, 0x2520_0C00 | 0x2520_1C00) => {
            Some(Opcode::SveWhileLo)
        }
        M::r#whilels if matches!(raw & 0xFF20_FC10, 0x2520_0C10 | 0x2520_1C10) => {
            Some(Opcode::SveWhileLs)
        }
        M::r#ptrues if (raw & 0xFF3F_FC10) == 0x2519_E000 => Some(Opcode::SvePtrues),
        _ => None,
    }
}
