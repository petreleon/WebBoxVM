use super::*;

pub(super) fn decode(raw: u32) -> DecodeStep {
    if (raw & 0xBF3F_FC00) == 0x0E31_B800 {
        let element_size = 1u64 << ((raw >> 22) & 0x3);
        return DecodeStep::Hit(reduce_instr(raw, Opcode::SimdAddv, element_size));
    }
    for (base, op) in [
        (0x6E30_F800, Opcode::SimdFpFmaxv),
        (0x6EB0_F800, Opcode::SimdFpFminv),
        (0x6E30_C800, Opcode::SimdFpFmaxnmv),
        (0x6EB0_C800, Opcode::SimdFpFminnmv),
    ] {
        if (raw & 0xFFFF_FC00) == base {
            return DecodeStep::Hit(reduce_instr(raw, op, 4));
        }
    }
    for (base, op) in [
        (0x0E30_A800, Opcode::SimdSmaxv),
        (0x0E31_A800, Opcode::SimdSminv),
        (0x2E30_A800, Opcode::SimdUmaxv),
        (0x2E31_A800, Opcode::SimdUminv),
    ] {
        if let Some(step) = decode_minmaxv(raw, base, op) {
            return step;
        }
    }
    DecodeStep::Miss
}

fn decode_minmaxv(raw: u32, base: u32, op: Opcode) -> Option<DecodeStep> {
    if (raw & 0xBF3F_FC00) != base {
        return None;
    }
    let q = ((raw >> 30) & 1) != 0;
    let element_size = 1u64 << ((raw >> 22) & 0x3);
    if element_size == 8 || (element_size == 4 && !q) {
        return Some(DecodeStep::Reject);
    }
    Some(DecodeStep::Hit(reduce_instr(raw, op, element_size)))
}

fn reduce_instr(raw: u32, op: Opcode, element_size: u64) -> Instr {
    Instr {
        op,
        rd: (raw & 0x1F) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm: 0,
        imm: element_size,
        sf: true,
        cond: 0,
        size: if (raw >> 30) != 0 { 16 } else { 8 },
    }
}
