use super::*;

pub(super) fn decode(raw: u32) -> Option<Instr> {
    let op = match raw & 0xFFE0_FC00 {
        0x9AC0_0000 => Opcode::Subp,
        0xBAC0_0000 => Opcode::Subps,
        _ => return None,
    };
    Some(Instr {
        op,
        rd: (raw & 0x1F) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm: ((raw >> 16) & 0x1F) as u8,
        imm: 0,
        sf: true,
        cond: 0,
        size: 0,
    })
}
