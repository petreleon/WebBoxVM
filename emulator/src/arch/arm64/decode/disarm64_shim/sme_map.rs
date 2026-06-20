use super::*;

pub(super) fn map(raw: u32, m: disarm64::decoder::Mnemonic) -> Option<Opcode> {
    use disarm64::decoder::Mnemonic as M;
    Some(match m {
        M::r#ldr if za_array(raw) => Opcode::SmeLdrZa,
        M::r#smlal if sme_smlal_indexed(raw) => Opcode::SmeSmlal,
        M::r#str if str_za_array(raw) => Opcode::SmeStrZa,
        _ => return None,
    })
}

fn za_array(raw: u32) -> bool {
    (raw & 0xFFFF_9C10) == 0xE100_0000
}

fn str_za_array(raw: u32) -> bool {
    (raw & 0xFFFF_9C10) == 0xE120_0000
}

fn sme_smlal_indexed(raw: u32) -> bool {
    (raw & 0xFFF0_9038) == 0xC1D0_1000 || (raw & 0xFFF0_9078) == 0xC1D0_9000
}
