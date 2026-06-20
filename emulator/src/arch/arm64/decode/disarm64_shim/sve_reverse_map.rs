use super::*;

pub(super) fn map(raw: u32, m: disarm64::decoder::Mnemonic) -> Option<Opcode> {
    use disarm64::decoder::Mnemonic as M;
    Some(match m {
        M::r#revh if revh(raw) => Opcode::SveRevh,
        _ => return None,
    })
}

fn revh(raw: u32) -> bool {
    ((raw >> 22) & 0x3) >= 2 && (raw & 0xFF3F_E000) == 0x0525_8000
}
