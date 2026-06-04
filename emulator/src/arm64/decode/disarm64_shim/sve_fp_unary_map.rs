use super::*;

pub(super) fn map(raw: u32, m: disarm64::decoder::Mnemonic) -> Option<Opcode> {
    use disarm64::decoder::Mnemonic as M;
    Some(match m {
        M::r#frintn if sve_fp_unary(raw, 0x6500_A000) => Opcode::SveFpFrintn,
        M::r#frintz if sve_fp_unary(raw, 0x6503_A000) => Opcode::SveFpFrintz,
        M::r#frinta if sve_fp_unary(raw, 0x6504_A000) => Opcode::SveFpFrinta,
        M::r#fsqrt if sve_fp_unary(raw, 0x650D_A000) => Opcode::SveFpSqrt,
        _ => return None,
    })
}

fn sve_fp_unary(raw: u32, base: u32) -> bool {
    ((raw >> 22) & 0x3) >= 2 && (raw & 0xFF3F_E000) == base
}
