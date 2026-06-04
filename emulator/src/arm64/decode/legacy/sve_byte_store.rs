use super::*;

pub(super) fn decode(raw: u32) -> DecodeStep {
    if (raw & 0xFF90_E000) != 0xE400_E000 {
        return DecodeStep::Miss;
    }
    let signed_imm = (((((raw >> 16) & 0xF) as i32) << 28) >> 28) as i64;
    DecodeStep::Hit(Instr {
        op: Opcode::SveSt1b,
        rd: (raw & 0x1F) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm: 0xFF,
        imm: signed_imm as u64,
        sf: true,
        cond: ((raw >> 10) & 0x7) as u8,
        size: 1u8 << (((raw >> 21) & 0x3) as u8),
    })
}
