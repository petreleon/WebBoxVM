use super::*;

pub(super) fn map(raw: u32, m: disarm64::decoder::Mnemonic) -> Option<Opcode> {
    use disarm64::decoder::Mnemonic as M;
    Some(match m {
        M::r#abs if scalar_unary(raw, 0x5AC0_2000) => Opcode::Abs,
        M::r#ctz if scalar_unary(raw, 0x5AC0_1800) => Opcode::Ctz,
        M::r#cnt if scalar_unary(raw, 0x5AC0_1C00) => Opcode::Cnt,
        M::r#smax if minmax(raw, 0x1AC0_6000, 0x11C0_0000) => Opcode::Smax,
        M::r#smin if minmax(raw, 0x1AC0_6800, 0x11C8_0000) => Opcode::Smin,
        M::r#umax if minmax(raw, 0x1AC0_6400, 0x11C4_0000) => Opcode::Umax,
        M::r#umin if minmax(raw, 0x1AC0_6C00, 0x11CC_0000) => Opcode::Umin,
        _ => return None,
    })
}

fn scalar_unary(raw: u32, base: u32) -> bool {
    (raw & 0x7FFF_FC00) == base
}

fn minmax(raw: u32, reg_base: u32, imm_base: u32) -> bool {
    (raw & 0x7FE0_FC00) == reg_base || (raw & 0x7FFC_0000) == imm_base
}
