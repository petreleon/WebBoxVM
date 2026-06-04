use super::*;

pub(super) fn decode(raw: u32) -> DecodeStep {
    if let Some(instr) = decode_smlal(raw, 4) {
        return DecodeStep::Hit(instr);
    }
    if let Some(instr) = decode_smlal(raw, 2) {
        return DecodeStep::Hit(instr);
    }
    DecodeStep::Miss
}

fn decode_smlal(raw: u32, nreg: u8) -> Option<Instr> {
    let matches = match nreg {
        2 => (raw & 0xFFF0_9038) == 0xC1D0_1000,
        4 => (raw & 0xFFF0_9078) == 0xC1D0_9000,
        _ => false,
    };
    if !matches {
        return None;
    }
    let index = ((((raw >> 10) & 0x3) << 1) | ((raw >> 2) & 1)) as u64;
    Some(Instr {
        op: Opcode::SmeSmlal,
        rd: (raw & 0x3) as u8,
        rn: sme_zn_base(raw, nreg),
        rm: ((raw >> 16) & 0xF) as u8,
        imm: index,
        sf: true,
        cond: ((raw >> 13) & 0x3) as u8,
        size: nreg,
    })
}

fn sme_zn_base(raw: u32, nreg: u8) -> u8 {
    match nreg {
        2 => (((raw >> 6) & 0xF) << 1) as u8,
        4 => (((raw >> 7) & 0x7) << 2) as u8,
        _ => 0,
    }
}
