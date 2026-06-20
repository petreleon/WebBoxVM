use super::*;

pub(super) fn decode(raw: u32) -> DecodeStep {
    if let Some(op) = zero_compare_op(raw) {
        return decode_compare(raw, op, 1, 0);
    }
    if let Some(op) = vector_compare_op(raw) {
        return decode_compare(raw, op, 0, ((raw >> 16) & 0x1F) as u8);
    }
    DecodeStep::Miss
}

fn vector_compare_op(raw: u32) -> Option<Opcode> {
    Some(match raw & 0xFF20_E010 {
        0x6500_4000 => Opcode::SveFpFcmge,
        0x6500_4010 => Opcode::SveFpFcmgt,
        0x6500_6000 => Opcode::SveFpFcmeq,
        0x6500_6010 => Opcode::SveFpFcmne,
        0x6500_C010 => Opcode::SveFpFacge,
        0x6500_E010 => Opcode::SveFpFacgt,
        _ => return None,
    })
}

fn zero_compare_op(raw: u32) -> Option<Opcode> {
    let op = match raw & 0xFF3F_E010 {
        0x6510_2000 => Opcode::SveFpFcmge,
        0x6510_2010 => Opcode::SveFpFcmgt,
        0x6511_2000 => Opcode::SveFpFcmlt,
        0x6511_2010 => Opcode::SveFpFcmle,
        0x6512_2000 => Opcode::SveFpFcmeq,
        0x6513_2000 => Opcode::SveFpFcmne,
        _ => return None,
    };
    Some(op)
}

fn decode_compare(raw: u32, op: Opcode, imm: u64, rm: u8) -> DecodeStep {
    let size = 1u8 << (((raw >> 22) & 0x3) as u8);
    if size == 1 {
        return DecodeStep::Reject;
    }
    DecodeStep::Hit(Instr {
        op,
        rd: (raw & 0xF) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm,
        imm,
        sf: true,
        cond: ((raw >> 10) & 0x7) as u8,
        size,
    })
}
