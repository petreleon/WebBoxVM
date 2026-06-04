use super::*;

pub(super) fn map(raw: u32, m: disarm64::decoder::Mnemonic) -> Option<Opcode> {
    use disarm64::decoder::Mnemonic as M;
    match m {
        M::r#index if (raw & 0xFF20_FC00) == 0x0420_4000 => Some(Opcode::SveIndex),
        M::r#index if (raw & 0xFF20_FC00) == 0x0420_4400 => Some(Opcode::SveIndex),
        M::r#index if (raw & 0xFF20_FC00) == 0x0420_4800 => Some(Opcode::SveIndex),
        M::r#index if (raw & 0xFF20_FC00) == 0x0420_4C00 => Some(Opcode::SveIndex),
        _ => None,
    }
}
