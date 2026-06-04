use super::*;

pub(super) fn decode(raw: u32) -> DecodeStep {
    for (base, op) in [
        (0x0E20_8800, Opcode::SimdCmgtZero),
        (0x0E20_9800, Opcode::SimdCmeqZero),
        (0x0E20_A800, Opcode::SimdCmltZero),
        (0x2E20_8800, Opcode::SimdCmgeZero),
        (0x2E20_9800, Opcode::SimdCmleZero),
    ] {
        if let Some(step) = decode_vector_compare_zero(raw, base, op) {
            return step;
        }
    }
    for (base, op) in [
        (0x5E20_8800, Opcode::SimdCmgtZero),
        (0x5E20_9800, Opcode::SimdCmeqZero),
        (0x5E20_A800, Opcode::SimdCmltZero),
        (0x7E20_8800, Opcode::SimdCmgeZero),
        (0x7E20_9800, Opcode::SimdCmleZero),
    ] {
        if let Some(step) = decode_scalar_compare_zero(raw, base, op) {
            return step;
        }
    }
    DecodeStep::Miss
}

fn decode_vector_compare_zero(raw: u32, base: u32, op: Opcode) -> Option<DecodeStep> {
    if (raw & 0xBF3F_FC00) != base {
        return None;
    }
    let q = ((raw >> 30) & 1) != 0;
    let element_size = 1u64 << ((raw >> 22) & 0x3);
    if element_size == 8 && !q {
        return Some(DecodeStep::Reject);
    }
    Some(compare_zero_instr(
        raw,
        op,
        element_size,
        if q { 16 } else { 8 },
    ))
}

fn decode_scalar_compare_zero(raw: u32, base: u32, op: Opcode) -> Option<DecodeStep> {
    if (raw & 0xFF3F_FC00) != base {
        return None;
    }
    if ((raw >> 22) & 0x3) != 0x3 {
        return Some(DecodeStep::Reject);
    }
    Some(compare_zero_instr(raw, op, 8, 8))
}

fn compare_zero_instr(raw: u32, op: Opcode, element_size: u64, vector_size: u8) -> DecodeStep {
    DecodeStep::Hit(Instr {
        op,
        rd: (raw & 0x1F) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm: 0,
        imm: element_size,
        sf: true,
        cond: 0,
        size: vector_size,
    })
}
