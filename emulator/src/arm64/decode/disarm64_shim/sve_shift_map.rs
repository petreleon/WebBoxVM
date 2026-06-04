use super::*;

pub(super) fn map(raw: u32, m: disarm64::decoder::Mnemonic) -> Option<Opcode> {
    use disarm64::decoder::Mnemonic as M;
    if !is_sve_shift_immediate(raw) {
        return None;
    }
    match m {
        M::r#asr => Some(Opcode::SveAsrImm),
        M::r#lsr => Some(Opcode::SveLsrImm),
        M::r#lsl => Some(Opcode::SveLslImm),
        _ => None,
    }
}

fn is_sve_shift_immediate(raw: u32) -> bool {
    let unpred = (raw & 0xFF20_F000) == 0x0420_9000 && ((raw >> 10) & 0x3) != 2;
    let pred = (raw & 0xFF3C_E000) == 0x0400_8000 && ((raw >> 16) & 0x3) != 2;
    (unpred || pred) && tsize(raw, pred) != 0
}

fn tsize(raw: u32, pred: bool) -> u32 {
    let low = if pred {
        (raw >> 8) & 0x3
    } else {
        (raw >> 19) & 0x3
    };
    (((raw >> 22) & 0x3) << 2) | low
}
