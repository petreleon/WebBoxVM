use super::*;

pub(in crate::arm64::decode) fn decode_simd_rcpc(raw: u32) -> Option<Instr> {
    let op = match raw & 0x3F60_0C00 {
        0x1D40_0800 => Opcode::SimdLdr,
        0x1D00_0800 => Opcode::SimdStr,
        _ => return None,
    };
    let size = rcpc3_size(raw)?;

    Some(Instr {
        op,
        rd: (raw & 0x1F) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm: 0xFF,
        imm: simm9(raw) as u64,
        sf: size >= 8,
        cond: 0,
        size,
    })
}

pub(in crate::arm64::decode) fn decode_simd_rcpc_lane(raw: u32) -> Option<Instr> {
    let op = match raw & 0xBFFF_FC00 {
        0x0D41_8400 => Opcode::SimdLd1Lane,
        0x0D01_8400 => Opcode::SimdSt1Lane,
        _ => return None,
    };

    Some(Instr {
        op,
        rd: (raw & 0x1F) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm: 0xFF,
        imm: ((raw >> 30) & 1) as u64,
        sf: true,
        cond: 8,
        size: 8,
    })
}

fn rcpc3_size(raw: u32) -> Option<u8> {
    match ((raw >> 23) & 1, (raw >> 30) & 3) {
        (0, size) => Some(1u8 << size),
        (1, 0) => Some(16),
        _ => None,
    }
}

fn simm9(raw: u32) -> i64 {
    let imm = ((raw >> 12) & 0x1FF) as i64;
    if imm & 0x100 != 0 {
        imm - 0x200
    } else {
        imm
    }
}
