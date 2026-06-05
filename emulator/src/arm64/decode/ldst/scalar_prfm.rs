use super::*;

pub(in crate::arm64::decode::ldst) fn decode_prfm(raw: u32) -> Option<Instr> {
    let bit24 = (raw >> 24) & 1;
    let bit21 = (raw >> 21) & 1;
    let bits11_10 = (raw >> 10) & 3;
    let rn = ((raw >> 5) & 0x1F) as u8;
    let rt = (raw & 0x1F) as u8;
    let simm9 = ((raw >> 12) & 0x1FF) as i64;
    let simm9 = if simm9 & 0x100 != 0 {
        simm9 - 0x200
    } else {
        simm9
    };
    let (rm, imm, cond) = if bit24 == 1 {
        (0xFF, ((raw >> 10) & 0xFFF) as u64 * 8, 0)
    } else if bit21 == 0 && bits11_10 == 0 {
        (0xFF, simm9 as u64, 1)
    } else if bit21 == 1 && bits11_10 == 2 {
        (
            ((raw >> 16) & 0x1F) as u8,
            ((raw >> 12) & 1) as u64,
            ((raw >> 13) & 7) as u8,
        )
    } else {
        return None;
    };
    Some(Instr {
        size: 0,
        op: Opcode::Prfm,
        rd: rt,
        rn,
        rm,
        imm,
        sf: true,
        cond,
    })
}
