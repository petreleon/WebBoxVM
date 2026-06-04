use super::*;

pub(super) fn decode(raw: u32) -> DecodeStep {
    if (raw & 0xFF38_FC00) != 0x6510_8000 {
        return DecodeStep::Miss;
    }

    let size_bits = ((raw >> 22) & 0x3) as u8;
    if size_bits == 0 {
        return DecodeStep::Reject;
    }

    let rd = (raw & 0x1F) as u8;
    DecodeStep::Hit(Instr {
        op: Opcode::SveFpFtmad,
        rd,
        rn: rd,
        rm: ((raw >> 5) & 0x1F) as u8,
        imm: ((raw >> 16) & 0x7) as u64,
        sf: true,
        cond: 0xFF,
        size: 1u8 << size_bits,
    })
}
