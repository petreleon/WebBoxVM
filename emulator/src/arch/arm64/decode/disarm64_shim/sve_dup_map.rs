use super::*;

pub(super) fn map(raw: u32, m: disarm64::decoder::Mnemonic) -> Option<Opcode> {
    use disarm64::decoder::Mnemonic as M;
    match m {
        M::r#dup if (raw & 0xFF3F_C000) == 0x2538_C000 => Some(Opcode::SveDupImm),
        M::r#dup if (raw & 0xFF20_FC00) == 0x0520_2000 => Some(Opcode::SveDupElem),
        M::r#cpy if (raw & 0xFF30_8000) == 0x0510_0000 => Some(Opcode::SveCpyImm),
        M::r#mov if (raw & 0xFF30_8000) == 0x0510_0000 => Some(Opcode::SveCpyImm),
        M::r#cpy if (raw & 0xFF3F_E000) == 0x0528_A000 => Some(Opcode::SveCpyGpr),
        _ => None,
    }
}
