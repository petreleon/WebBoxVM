use super::*;

pub(in crate::arch::arm64::decode) fn decode_ldrauth(raw: u32) -> Option<Instr> {
    let op = match raw & 0xFFA0_0400 {
        0xF820_0400 => Opcode::Ldraa,
        0xF8A0_0400 => Opcode::Ldrab,
        _ => return None,
    };
    let imm10 = ((raw >> 12) & 0x1FF) | (((raw >> 22) & 1) << 9);
    let simm = if (imm10 & 0x200) != 0 {
        imm10 as i64 - 0x400
    } else {
        imm10 as i64
    };
    Some(Instr {
        op,
        rd: (raw & 0x1F) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm: 0xFF,
        imm: (simm * 8) as u64,
        sf: true,
        cond: if ((raw >> 11) & 1) != 0 { 3 } else { 0 },
        size: 8,
    })
}
