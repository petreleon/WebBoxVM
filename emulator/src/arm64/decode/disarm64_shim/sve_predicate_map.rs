use super::*;

pub(super) fn map(raw: u32, m: disarm64::decoder::Mnemonic) -> Option<Opcode> {
    use disarm64::decoder::Mnemonic as M;
    match m {
        M::r#whilelo if (raw & 0xFF20_EC10) == 0x2520_0C00 => Some(Opcode::SveWhileLo),
        M::r#ptrues if (raw & 0xFF3F_FC10) == 0x2519_E000 => Some(Opcode::SvePtrues),
        _ => None,
    }
}
