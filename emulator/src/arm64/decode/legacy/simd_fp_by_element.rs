use super::*;

pub(super) fn decode(raw: u32) -> DecodeStep {
    let op = match raw & 0xBF00_F400 {
        0x0F00_1000 => Opcode::SimdFpFmlaElem,
        0x0F00_5000 => Opcode::SimdFpFmlsElem,
        0x0F00_9000 => Opcode::SimdFpMulElem,
        _ => return DecodeStep::Miss,
    };
    let q = ((raw >> 30) & 1) != 0;
    let sz = ((raw >> 22) & 1) != 0;
    let l = ((raw >> 21) & 1) as u8;

    if sz && (!q || l != 0) {
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
        size: if q { 16 } else { 8 },
    })
}
