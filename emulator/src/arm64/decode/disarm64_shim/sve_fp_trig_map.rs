use super::*;

pub(super) fn map(raw: u32, m: disarm64::decoder::Mnemonic) -> Option<Opcode> {
    use disarm64::decoder::Mnemonic as M;
    Some(match m {
        M::r#ftmad if (raw & 0xFF38_FC00) == 0x6510_8000 => Opcode::SveFpFtmad,
        _ => return None,
    })
}
