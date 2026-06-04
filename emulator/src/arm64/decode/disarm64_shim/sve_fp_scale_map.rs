use super::*;

pub(super) fn map(raw: u32, m: disarm64::decoder::Mnemonic) -> Option<Opcode> {
    use disarm64::decoder::Mnemonic as M;
    Some(match m {
        M::r#fscale if (raw & 0xFF3F_E000) == 0x6509_8000 => Opcode::SveFpFscale,
        _ => return None,
    })
}
