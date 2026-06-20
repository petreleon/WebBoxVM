use super::*;
use crate::arch::arm64::bitmask_imm::decode_bitmask_imm;

pub(super) fn decode(raw: u32) -> DecodeStep {
    let op = match raw & 0xFFFC_0000 {
        0x0500_0000 => Opcode::SveOrrImm,
        0x0540_0000 => Opcode::SveEorImm,
        0x0580_0000 => Opcode::SveAndImm,
        0x05C0_0000 => Opcode::SveDupm,
        _ => return DecodeStep::Miss,
    };

    let imm13 = (raw >> 5) & 0x1FFF;
    let n = (imm13 >> 12) & 1;
    let imms = imm13 & 0x3F;
    let immr = (imm13 >> 6) & 0x3F;
    let Some(imm) = decode_bitmask_imm(n, immr, imms, true) else {
        return DecodeStep::Reject;
    };

    DecodeStep::Hit(Instr {
        op,
        rd: (raw & 0x1F) as u8,
        rn: (raw & 0x1F) as u8,
        rm: 0,
        imm,
        sf: true,
        cond: 0xFF,
        size: 8,
    })
}
