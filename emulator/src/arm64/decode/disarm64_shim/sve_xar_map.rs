use super::*;

pub(super) fn map(raw: u32, m: disarm64::decoder::Mnemonic) -> Option<Opcode> {
    use disarm64::decoder::Mnemonic as M;
    match m {
        M::r#xar if (raw & 0xFF20_FC00) == 0x0420_3400 => Some(Opcode::SveXar),
        _ => None,
    }
}
