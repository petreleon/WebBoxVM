use super::*;

pub(super) fn map(raw: u32, m: disarm64::decoder::Mnemonic) -> Option<Opcode> {
    let name = format!("{m:?}");
    if is_cpyf(raw) && name.starts_with("cpyf") {
        return Some(cpyf_opcode((raw >> 22) & 0x3));
    }
    if is_cpy(raw) && name.starts_with("cpy") {
        return Some(cpy_opcode((raw >> 22) & 0x3));
    }
    if is_set(raw) && name.starts_with("set") {
        return Some(set_opcode((raw >> 14) & 0x3));
    }
    None
}

fn is_cpyf(raw: u32) -> bool {
    (raw & 0xFF20_0C00) == 0x1900_0400 && ((raw >> 22) & 0x3) < 3
}

fn is_cpy(raw: u32) -> bool {
    (raw & 0xFF20_0C00) == 0x1D00_0400 && ((raw >> 22) & 0x3) < 3
}

fn is_set(raw: u32) -> bool {
    (raw & 0xFF20_0C00) == 0x1900_0400 && ((raw >> 22) & 0x3) == 3
}

fn cpyf_opcode(stage: u32) -> Opcode {
    [Opcode::MopsCpyFp, Opcode::MopsCpyFm, Opcode::MopsCpyFe][stage as usize]
}

fn cpy_opcode(stage: u32) -> Opcode {
    [Opcode::MopsCpyP, Opcode::MopsCpyM, Opcode::MopsCpyE][stage as usize]
}

fn set_opcode(stage: u32) -> Opcode {
    [Opcode::MopsSetP, Opcode::MopsSetM, Opcode::MopsSetE][stage as usize]
}
