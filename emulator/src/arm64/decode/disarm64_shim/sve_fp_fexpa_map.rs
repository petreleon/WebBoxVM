use super::*;

pub(super) fn map(raw: u32, m: disarm64::decoder::Mnemonic) -> Option<Opcode> {
    use disarm64::decoder::Mnemonic as M;
    Some(match m {
        M::r#fexpa if ((raw >> 22) & 0x3) != 0 && (raw & 0xFF3F_FC00) == 0x0420_B800 => {
            Opcode::SveFpFexpa
        }
        _ => return None,
    })
}
