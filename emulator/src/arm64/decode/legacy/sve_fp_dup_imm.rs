use super::*;

pub(super) fn decode(raw: u32) -> Option<DecodeStep> {
    if (raw & 0xFF3F_C000) != 0x2539_C000 {
        return None;
    }

    let size = 1u8 << (((raw >> 22) & 0x3) as u8);
    if size == 1 {
        return Some(DecodeStep::Reject);
    }

    Some(DecodeStep::Hit(Instr {
        op: Opcode::SveFpDupImm,
        rd: (raw & 0x1F) as u8,
        rn: 0,
        rm: 0,
        imm: ((raw >> 5) & 0xFF) as u64,
        sf: true,
        cond: 0xFF,
        size,
    }))
}
