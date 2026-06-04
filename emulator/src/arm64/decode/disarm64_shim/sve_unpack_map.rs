use super::*;

pub(super) fn map(raw: u32, m: disarm64::decoder::Mnemonic) -> Option<Opcode> {
    use disarm64::decoder::Mnemonic as M;
    Some(match m {
        M::r#sunpklo if vector_unpack(raw, 0x0530_3800) => Opcode::SveSunpklo,
        M::r#sunpkhi if vector_unpack(raw, 0x0531_3800) => Opcode::SveSunpkhi,
        M::r#uunpklo if vector_unpack(raw, 0x0532_3800) => Opcode::SveUunpklo,
        M::r#uunpkhi if vector_unpack(raw, 0x0533_3800) => Opcode::SveUunpkhi,
        M::r#punpklo if (raw & 0xFFFF_FC10) == 0x0530_4000 => Opcode::SvePunpklo,
        M::r#punpkhi if (raw & 0xFFFF_FC10) == 0x0531_4000 => Opcode::SvePunpkhi,
        _ => return None,
    })
}

fn vector_unpack(raw: u32, base: u32) -> bool {
    ((raw >> 22) & 0x3) != 0 && (raw & 0xFF3F_FC00) == base
}
