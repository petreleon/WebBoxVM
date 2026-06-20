use super::*;

pub(super) fn decode(raw: u32) -> DecodeStep {
    let (op, scalar) = match raw & 0xBF80_F400 {
        0x0F80_1000 => (Opcode::SimdFpFmlaElem, false),
        0x0F80_5000 => (Opcode::SimdFpFmlsElem, false),
        0x0F80_9000 => (Opcode::SimdFpMulElem, false),
        0x2F80_9000 => (Opcode::SimdFpMulxElem, false),
        0x3F80_9000 => (Opcode::SimdFpMulxElem, true),
        _ => return DecodeStep::Miss,
    };
    let q = ((raw >> 30) & 1) != 0;
    let sz = ((raw >> 22) & 1) != 0;
    let l = ((raw >> 21) & 1) as u8;

    if (scalar && !q) || (sz && (l != 0 || (!scalar && !q))) {
        return DecodeStep::Reject;
    }

    let h = ((raw >> 11) & 1) as u8;
    let element_size = if sz { 8 } else { 4 };
    let lane = if sz { h } else { (h << 1) | l };
    DecodeStep::Hit(Instr {
        op,
        rd: (raw & 0x1F) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm: ((((raw >> 20) & 1) << 4) | ((raw >> 16) & 0xF)) as u8,
        imm: element_size,
        sf: true,
        cond: lane,
        size: if scalar {
            element_size as u8
        } else if q {
            16
        } else {
            8
        },
    })
}
