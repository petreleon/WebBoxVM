use super::*;

pub(super) fn decode(raw: u32) -> DecodeStep {
    if let Some(instr) = decode_scalar(raw) {
        return DecodeStep::Hit(instr);
    }
    if let Some(step) = decode_vector(raw) {
        return step;
    }
    DecodeStep::Miss
}

fn decode_scalar(raw: u32) -> Option<Instr> {
    for (base, op) in [
        (0x5EE0_4400, Opcode::SimdSshl),
        (0x7EE0_4400, Opcode::SimdUshl),
    ] {
        if (raw & 0xFFE0_FC00) == base {
            return Some(Instr {
                op,
                rd: (raw & 0x1F) as u8,
                rn: ((raw >> 5) & 0x1F) as u8,
                rm: ((raw >> 16) & 0x1F) as u8,
                imm: 8,
                sf: true,
                cond: 0,
                size: 8,
            });
        }
    }
    None
}

fn decode_vector(raw: u32) -> Option<DecodeStep> {
    for (base, op) in [
        (0x0E20_4400, Opcode::SimdSshl),
        (0x2E20_4400, Opcode::SimdUshl),
    ] {
        if (raw & 0xBF20_FC00) == base {
            let q = (raw >> 30) != 0;
            let element_size = 1u64 << ((raw >> 22) & 0x3);
            if element_size == 8 && !q {
                return Some(DecodeStep::Reject);
            }
            return Some(DecodeStep::Hit(Instr {
                op,
                rd: (raw & 0x1F) as u8,
                rn: ((raw >> 5) & 0x1F) as u8,
                rm: ((raw >> 16) & 0x1F) as u8,
                imm: element_size,
                sf: true,
                cond: 0,
                size: if q { 16 } else { 8 },
            }));
        }
    }
    None
}
