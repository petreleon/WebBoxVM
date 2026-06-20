use super::*;

pub(super) fn decode(raw: u32) -> Option<DecodeStep> {
    let op = if (raw & 0xFF3F_C000) == 0x2539_C000 {
        Opcode::SveFpDupImm
    } else if (raw & 0xFF30_E000) == 0x0510_C000 {
        Opcode::SveFpCpyImm
    } else {
        return None;
    };

    let size = 1u8 << (((raw >> 22) & 0x3) as u8);
    if size == 1 {
        return Some(DecodeStep::Reject);
    }

    Some(DecodeStep::Hit(Instr {
        op,
        rd: (raw & 0x1F) as u8,
        rn: 0,
        rm: 0,
        imm: ((raw >> 5) & 0xFF) as u64,
        sf: true,
        cond: if op == Opcode::SveFpCpyImm {
            ((raw >> 16) & 0xF) as u8
        } else {
            0xFF
        },
        size,
    }))
}
