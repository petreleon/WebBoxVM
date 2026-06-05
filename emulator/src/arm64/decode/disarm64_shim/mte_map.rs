use super::*;

pub(super) fn map(raw: u32, m: disarm64::decoder::Mnemonic) -> Option<Opcode> {
    use disarm64::decoder::Mnemonic as M;
    Some(match m {
        M::r#addg if (raw & 0xFFC0_C000) == 0x9180_0000 => Opcode::MteAddg,
        M::r#subg if (raw & 0xFFC0_C000) == 0xD180_0000 => Opcode::MteSubg,
        M::r#ldg if (raw & 0xFFE0_0C00) == 0xD960_0000 => Opcode::MteLdg,
        M::r#irg if (raw & 0xFFE0_FC00) == 0x9AC0_1000 => Opcode::MteIrg,
        M::r#gmi if (raw & 0xFFE0_FC00) == 0x9AC0_1400 => Opcode::MteGmi,
        M::r#stg if is_tag_store(raw, 0xD920_0000) => Opcode::MteStg,
        M::r#stgp if is_stgp(raw) => Opcode::MteStgp,
        M::r#stzg if is_tag_store(raw, 0xD960_0000) => Opcode::MteStzg,
        M::r#st2g if is_tag_store(raw, 0xD9A0_0000) => Opcode::MteSt2g,
        M::r#stz2g if is_tag_store(raw, 0xD9E0_0000) => Opcode::MteStz2g,
        _ => return None,
    })
}

fn is_tag_store(raw: u32, base: u32) -> bool {
    (raw & 0xFFE0_0000) == base && ((raw >> 10) & 0x3) != 0
}

fn is_stgp(raw: u32) -> bool {
    (raw & 0x7E40_0000) == 0x6800_0000 && ((raw >> 23) & 0x3) != 0 && ((raw >> 22) & 1) == 0
}
