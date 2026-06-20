use super::*;

pub(super) fn decode(raw: u32) -> DecodeStep {
    if (raw & 0xFF3F_FC00) != 0x0420_B800 {
        return DecodeStep::Miss;
    }

    let size_bits = ((raw >> 22) & 0x3) as u8;
    if size_bits == 0 {
        return DecodeStep::Reject;
    }

    DecodeStep::Hit(Instr {
        op: Opcode::SveFpFexpa,
        rd: (raw & 0x1F) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm: 0,
        imm: 0,
        sf: true,
        cond: 0xFF,
        size: 1u8 << size_bits,
    })
}
