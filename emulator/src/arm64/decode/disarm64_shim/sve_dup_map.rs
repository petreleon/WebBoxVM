use super::*;

pub(super) fn map(raw: u32, m: disarm64::decoder::Mnemonic) -> Option<Opcode> {
    use disarm64::decoder::Mnemonic as M;
    match m {
        M::r#dup if (raw & 0xFF3F_C000) == 0x2538_C000 => Some(Opcode::SveDupImm),
        M::r#dup if (raw & 0xFF20_FC00) == 0x0520_2000 => Some(Opcode::SveDupElem),
        _ => None,
    }
}
