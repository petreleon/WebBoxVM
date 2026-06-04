use super::*;

pub(super) fn map(raw: u32, m: disarm64::decoder::Mnemonic) -> Option<Opcode> {
    use disarm64::decoder::Mnemonic as M;
    match m {
        M::r#and if (raw & 0xFFE0_FC00) == 0x0420_3000 => Some(Opcode::SveAndVec),
        M::r#orr if (raw & 0xFF3F_E000) == 0x0418_0000 => Some(Opcode::SveOrrPred),
        M::r#eor if (raw & 0xFF3F_E000) == 0x0419_0000 => Some(Opcode::SveEorPred),
        M::r#and if (raw & 0xFF3F_E000) == 0x041A_0000 => Some(Opcode::SveAndPred),
        M::r#eor if (raw & 0xFFF0_C210) == 0x2500_4200 => Some(Opcode::SvePredEor),
        M::r#eors if (raw & 0xFFF0_C210) == 0x2540_4200 => Some(Opcode::SvePredEor),
        _ => None,
    }
}
