use super::*;

pub(super) fn map(raw: u32, m: disarm64::decoder::Mnemonic) -> Option<Opcode> {
    use disarm64::decoder::Mnemonic as M;
    match m {
        M::r#add if (raw & 0xFF3F_C000) == 0x2520_C000 => Some(Opcode::SveAddImm),
        M::r#sub if (raw & 0xFF3F_C000) == 0x2521_C000 => Some(Opcode::SveSubImm),
        M::r#add if (raw & 0xFF3F_E000) == 0x0400_0000 => Some(Opcode::SveAddPred),
        M::r#sub if (raw & 0xFF3F_E000) == 0x0401_0000 => Some(Opcode::SveSubPred),
        _ => None,
    }
}
