use super::*;

pub(super) fn decode(raw: u32) -> DecodeStep {
    if (raw & 0xFF3F_E000) != 0x6509_8000 {
        return DecodeStep::Miss;
    }

    let size_bits = ((raw >> 22) & 0x3) as u8;
    if size_bits == 0 {
        return DecodeStep::Reject;
    }

    let rd = (raw & 0x1F) as u8;
    DecodeStep::Hit(Instr {
        op: Opcode::SveFpFscale,
        rd,
        rn: rd,
        rm: ((raw >> 5) & 0x1F) as u8,
        imm: 0,
        sf: true,
        cond: ((raw >> 10) & 0x7) as u8,
        size: 1u8 << size_bits,
    })
}
