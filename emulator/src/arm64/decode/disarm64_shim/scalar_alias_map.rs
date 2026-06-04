use super::*;

pub(super) fn map(raw: u32, m: disarm64::decoder::Mnemonic) -> Option<Opcode> {
    use disarm64::decoder::Mnemonic as M;
    Some(match m {
        M::r#orr if legacy_mov_reg(raw) => Opcode::MovReg,
        M::r#sbfm if legacy_sxtw(raw) => Opcode::Sxtw,
        M::r#extr => Opcode::Extr,
        _ => return None,
    })
}

fn legacy_mov_reg(raw: u32) -> bool {
    (raw & 0x7FE0_FC00) == 0x2A00_0000 && ((raw >> 5) & 0x1F) == 31
}

fn legacy_sxtw(raw: u32) -> bool {
    let sf = (raw >> 31) & 1;
    let n = (raw >> 22) & 1;
    let opc = (raw >> 29) & 3;
    let immr = (raw >> 16) & 0x3F;
    let imms = (raw >> 10) & 0x3F;
    sf == n && opc == 0 && immr == 0 && imms == 31
}
