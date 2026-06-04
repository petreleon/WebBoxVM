use super::*;

pub(super) fn map(raw: u32, m: disarm64::decoder::Mnemonic) -> Option<Opcode> {
    use disarm64::decoder::Mnemonic as M;
    Some(match m {
        M::r#ftmad if (raw & 0xFF38_FC00) == 0x6510_8000 => Opcode::SveFpFtmad,
        M::r#ftsmul if sve_fp_trig_binary(raw, 0x6500_0C00) => Opcode::SveFpFtsmul,
        M::r#ftssel if sve_fp_trig_binary(raw, 0x0420_B000) => Opcode::SveFpFtssel,
        _ => return None,
    })
}

fn sve_fp_trig_binary(raw: u32, base: u32) -> bool {
    ((raw >> 22) & 0x3) != 0 && (raw & 0xFF20_FC00) == base
}
