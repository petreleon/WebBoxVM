use super::*;

pub(in crate::arm64::decode) fn decode_cssc_minmax_imm(raw: u32) -> Option<Instr> {
    let op = match raw & 0x7FFC_0000 {
        0x11C0_0000 => Opcode::Smax,
        0x11C8_0000 => Opcode::Smin,
        0x11C4_0000 => Opcode::Umax,
        0x11CC_0000 => Opcode::Umin,
        _ => return None,
    };
    let sf = ((raw >> 31) & 1) != 0;
    let imm8 = ((raw >> 10) & 0xFF) as u8;
    let signed = matches!(op, Opcode::Smax | Opcode::Smin);
    Some(Instr {
        size: 0,
        op,
        rd: (raw & 0x1F) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm: 0xFF,
        imm: if signed {
            (imm8 as i8 as i64) as u64
        } else {
            imm8 as u64
        },
        sf,
        cond: 0,
    })
}
