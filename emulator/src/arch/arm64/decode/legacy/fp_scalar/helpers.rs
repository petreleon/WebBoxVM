use super::*;

pub(super) fn fp_instr(op: Opcode, rd: u8, rn: u8, rm: u8, imm: u64, size: u8) -> Instr {
    Instr {
        op,
        rd,
        rn,
        rm,
        imm,
        sf: size == 8,
        cond: 0,
        size,
    }
}

pub(super) fn fp_scalar_type_size(ftype: u8) -> Option<u8> {
    match ftype {
        0 => Some(4),
        1 => Some(8),
        3 => Some(2),
        _ => None,
    }
}
