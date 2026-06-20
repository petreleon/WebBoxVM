use super::*;

pub(super) fn map(raw: u32, m: disarm64::decoder::Mnemonic) -> Option<Opcode> {
    use disarm64::decoder::Mnemonic as M;
    match m {
        M::r#and if (raw & 0xFFE0_FC00) == 0x0420_3000 => Some(Opcode::SveAndVec),
        M::r#orr if (raw & 0xFF3F_E000) == 0x0418_0000 => Some(Opcode::SveOrrPred),
        M::r#eor if (raw & 0xFF3F_E000) == 0x0419_0000 => Some(Opcode::SveEorPred),
        M::r#and if (raw & 0xFF3F_E000) == 0x041A_0000 => Some(Opcode::SveAndPred),
        M::r#bic if (raw & 0xFFF0_C210) == 0x2500_4010 => Some(Opcode::SvePredBic),
        M::r#bics if (raw & 0xFFF0_C210) == 0x2540_4010 => Some(Opcode::SvePredBic),
        M::r#eor if (raw & 0xFFF0_C210) == 0x2500_4200 => Some(Opcode::SvePredEor),
        M::r#eors if (raw & 0xFFF0_C210) == 0x2540_4200 => Some(Opcode::SvePredEor),
        M::r#orn if (raw & 0xFFF0_C210) == 0x2580_4010 => Some(Opcode::SvePredOrn),
        M::r#orns if (raw & 0xFFF0_C210) == 0x25C0_4010 => Some(Opcode::SvePredOrn),
        M::r#nor if (raw & 0xFFF0_C210) == 0x2580_4200 => Some(Opcode::SvePredNor),
        M::r#nors if (raw & 0xFFF0_C210) == 0x25C0_4200 => Some(Opcode::SvePredNor),
        M::r#nand if (raw & 0xFFF0_C210) == 0x2580_4210 => Some(Opcode::SvePredNand),
        M::r#nands if (raw & 0xFFF0_C210) == 0x25C0_4210 => Some(Opcode::SvePredNand),
        _ => None,
    }
}
