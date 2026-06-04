use super::*;

pub(super) fn decode(raw: u32) -> DecodeStep {
    if (raw & 0xFF20_FC00) != 0x0420_3400 {
        return DecodeStep::Miss;
    }
    let tsize = (((raw >> 22) & 0x3) << 2) | ((raw >> 19) & 0x3);
    let Some(size) = element_size(tsize) else {
        return DecodeStep::Reject;
    };
    let imm3 = (raw >> 16) & 0x7;
    let esize = (size as u64) * 8;
    let rot = (2 * esize) - (((tsize << 3) | imm3) as u64);

    DecodeStep::Hit(Instr {
        op: Opcode::SveXar,
        rd: (raw & 0x1F) as u8,
        rn: (raw & 0x1F) as u8,
        rm: ((raw >> 5) & 0x1F) as u8,
        imm: rot,
        sf: true,
        cond: 0xFF,
        size,
    })
}

fn element_size(tsize: u32) -> Option<u8> {
    (tsize != 0).then(|| 1u8 << (31 - tsize.leading_zeros()))
}
