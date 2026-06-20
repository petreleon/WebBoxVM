use super::*;

pub(super) fn decode(raw: u32) -> Option<DecodeStep> {
    let op = match raw & 0xFF3F_E3C0 {
        0x6518_8000 => Opcode::SveFpAddImm,
        0x6519_8000 => Opcode::SveFpSub,
        0x651A_8000 => Opcode::SveFpMulImm,
        0x651B_8000 => Opcode::SveFpSubr,
        _ => return None,
    };
    let size = 1u8 << (((raw >> 22) & 0x3) as u8);
    if size == 1 {
        return Some(DecodeStep::Reject);
    }
    let rd = (raw & 0x1F) as u8;
    Some(DecodeStep::Hit(Instr {
        op,
        rd,
        rn: rd,
        rm: 0xFF,
        imm: ((raw >> 5) & 1) as u64,
        sf: true,
        cond: ((raw >> 10) & 0x7) as u8,
        size,
    }))
}
