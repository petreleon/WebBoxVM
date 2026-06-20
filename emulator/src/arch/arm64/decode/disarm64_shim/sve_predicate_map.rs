use super::*;

pub(super) fn map(raw: u32, m: disarm64::decoder::Mnemonic) -> Option<Opcode> {
    use disarm64::decoder::Mnemonic as M;
    match m {
        M::r#whilege if matches!(raw & 0xFF20_FC10, 0x2520_0000 | 0x2520_1000) => {
            Some(Opcode::SveWhileGe)
        }
        M::r#whilegt if matches!(raw & 0xFF20_FC10, 0x2520_0010 | 0x2520_1010) => {
            Some(Opcode::SveWhileGt)
        }
        M::r#whilelt if matches!(raw & 0xFF20_FC10, 0x2520_0400 | 0x2520_1400) => {
            Some(Opcode::SveWhileLt)
        }
        M::r#whilele if matches!(raw & 0xFF20_FC10, 0x2520_0410 | 0x2520_1410) => {
            Some(Opcode::SveWhileLe)
        }
        M::r#whilehs if matches!(raw & 0xFF20_FC10, 0x2520_0800 | 0x2520_1800) => {
            Some(Opcode::SveWhileHs)
        }
        M::r#whilehi if matches!(raw & 0xFF20_FC10, 0x2520_0810 | 0x2520_1810) => {
            Some(Opcode::SveWhileHi)
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
