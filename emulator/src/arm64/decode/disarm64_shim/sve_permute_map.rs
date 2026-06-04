use super::*;

pub(super) fn map(raw: u32, m: disarm64::decoder::Mnemonic) -> Option<Opcode> {
    use disarm64::decoder::Mnemonic as M;
    Some(match m {
        M::r#zip1 if vector_zip(raw, 0x0520_6000) => Opcode::SveZip1,
        M::r#zip2 if vector_zip(raw, 0x0520_6400) => Opcode::SveZip2,
        M::r#tbl if vector_tbl(raw) => Opcode::SveTbl,
        _ => return None,
    })
}

fn vector_zip(raw: u32, base: u32) -> bool {
    (raw & 0xFF20_FC00) == base
}

fn vector_tbl(raw: u32) -> bool {
    matches!(raw & 0xFF20_FC00, 0x0520_3000 | 0x0520_2800)
}
