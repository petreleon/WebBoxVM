use super::*;

pub(super) fn map(raw: u32, m: disarm64::decoder::Mnemonic) -> Option<Opcode> {
    use disarm64::decoder::Mnemonic as M;
    Some(match m {
        M::r#rbit if scalar_sf(raw, 0x5AC0_0000) => Opcode::Rbit,
        M::r#rev16 if scalar_sf(raw, 0x5AC0_0400) => Opcode::Rev16,
        M::r#rev if rev(raw) => Opcode::Rev,
        M::r#rev32 if (raw & 0xFFFF_FC00) == 0xDAC0_0800 => Opcode::Rev32,
        M::r#clz if scalar_sf(raw, 0x5AC0_1000) => Opcode::Clz,
        M::r#cls if scalar_sf(raw, 0x5AC0_1400) => Opcode::Cls,
        M::r#ctz if scalar_sf(raw, 0x5AC0_1800) => Opcode::Ctz,
        M::r#cnt if scalar_sf(raw, 0x5AC0_1C00) => Opcode::Cnt,
        M::r#abs if scalar_sf(raw, 0x5AC0_2000) => Opcode::Abs,
        _ => return None,
    })
}

fn scalar_sf(raw: u32, base: u32) -> bool {
    (raw & 0x7FFF_FC00) == base
}

fn rev(raw: u32) -> bool {
    matches!(raw & 0xFFFF_FC00, 0x5AC0_0800 | 0xDAC0_0C00)
}
