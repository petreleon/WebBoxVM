use super::*;

pub(super) fn decode(raw: u32) -> DecodeStep {
    if (raw & 0xFF3F_E000) != 0x0525_8000 {
        return DecodeStep::Miss;
    }
    let size = 1u8 << (((raw >> 22) & 0x3) as u8);
    if size < 4 {
        return DecodeStep::Reject;
    }

    DecodeStep::Hit(Instr {
        op: Opcode::SveRevh,
        rd: (raw & 0x1F) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm: 0,
        imm: 2,
        sf: true,
        cond: ((raw >> 10) & 0x7) as u8,
        size,
    })
}
