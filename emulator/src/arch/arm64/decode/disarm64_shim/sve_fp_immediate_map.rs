use super::*;

pub(super) fn map(raw: u32, m: disarm64::decoder::Mnemonic) -> Option<Opcode> {
    use disarm64::decoder::Mnemonic as M;
    Some(match m {
        M::r#fcpy if (raw & 0xFF30_E000) == 0x0510_C000 => Opcode::SveFpCpyImm,
        _ => return None,
    })
}
