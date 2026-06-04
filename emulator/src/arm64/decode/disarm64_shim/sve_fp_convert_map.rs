use super::*;

pub(super) fn map(raw: u32, m: disarm64::decoder::Mnemonic) -> Option<Opcode> {
    use disarm64::decoder::Mnemonic as M;
    Some(match m {
        M::r#scvtf if sve_fp_convert(raw) => Opcode::SveScvtf,
        M::r#fcvtzs if sve_fp_convert(raw) => Opcode::SveFcvtzs,
        _ => return None,
    })
}

fn sve_fp_convert(raw: u32) -> bool {
    matches!(
        raw & 0xFFFF_E000,
        0x6594_A000
            | 0x65D0_A000
            | 0x65D4_A000
            | 0x65D6_A000
            | 0x659C_A000
            | 0x65DC_A000
            | 0x65D8_A000
            | 0x65DE_A000
    )
}
