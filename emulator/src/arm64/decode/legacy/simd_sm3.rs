use super::*;

pub(super) fn decode(raw: u32) -> DecodeStep {
    if (raw & 0xFFe0_8000) == 0xCE40_0000 {
        return DecodeStep::Hit(sm3_instr(
            raw,
            Opcode::SimdSm3Ss1,
            0,
            ((raw >> 10) & 0x1F) as u8,
        ));
    }
    if let Some(op) = sm3_tt(raw) {
        return DecodeStep::Hit(sm3_instr(raw, op, ((raw >> 12) & 0x3) as u64, 0));
    }
    match raw & 0xFFE0_FC00 {
        0xCE60_C000 => DecodeStep::Hit(sm3_instr(raw, Opcode::SimdSm3Partw1, 0, 0)),
        0xCE60_C400 => DecodeStep::Hit(sm3_instr(raw, Opcode::SimdSm3Partw2, 0, 0)),
        _ => DecodeStep::Miss,
    }
}

fn sm3_tt(raw: u32) -> Option<Opcode> {
    match raw & 0xFFE0_CC00 {
        0xCE40_8000 => Some(Opcode::SimdSm3Tt1A),
        0xCE40_8400 => Some(Opcode::SimdSm3Tt1B),
        0xCE40_8800 => Some(Opcode::SimdSm3Tt2A),
        0xCE40_8C00 => Some(Opcode::SimdSm3Tt2B),
        _ => None,
    }
}

fn sm3_instr(raw: u32, op: Opcode, imm: u64, cond: u8) -> Instr {
    Instr {
        op,
        rd: (raw & 0x1F) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm: ((raw >> 16) & 0x1F) as u8,
        imm,
        sf: true,
        cond,
        size: 16,
    }
}
