use super::*;

pub(super) fn decode_simd_bic_imm(raw: u32) -> Option<Instr> {
    let rd = (raw & 0x1F) as u8;
    let imm8 = (((raw >> 16) & 0x7) << 5) | ((raw >> 5) & 0x1F);
    let q = (raw >> 30) & 1;
    let cmode = (raw >> 12) & 0xF;

    if (raw & 0xBFF8_DC00) == 0x2F00_9400 {
        let shift = ((cmode >> 1) & 1) * 8;
        return Some(Instr {
            op: Opcode::SimdBicImm,
            rd,
            rn: 0,
            rm: 0,
            imm: (imm8 << shift) as u64,
            sf: true,
            cond: 2,
            size: if q == 1 { 16 } else { 8 },
        });
    }

    if (raw & 0xBFF8_9C00) == 0x2F00_1400 {
        let shift = ((cmode >> 1) & 0x3) * 8;
        return Some(Instr {
            op: Opcode::SimdBicImm,
            rd,
            rn: 0,
            rm: 0,
            imm: (imm8 << shift) as u64,
            sf: true,
            cond: 4,
            size: if q == 1 { 16 } else { 8 },
        });
    }

    None
}

pub(super) fn decode_simd_shl(raw: u32) -> Option<Instr> {
    let vector = (raw & 0xBF80_FC00) == 0x0F00_5400;
    let scalar = (raw & 0xFF80_FC00) == 0x5F00_5400;
    if !vector && !scalar {
        return None;
    }

    let immh = ((raw >> 19) & 0xF) as u8;
    if immh == 0 {
        return None;
    }
    let immb = ((raw >> 16) & 0x7) as u8;
    let highest = 7 - immh.leading_zeros() as u8;
    let element_size = 1u8 << highest;
    let imm = ((immh as u16) << 3) | immb as u16;
    let element_bits = element_size as u16 * 8;
    let shift = imm.checked_sub(element_bits)? as u64;

    Some(Instr {
        op: Opcode::SimdShlImm,
        rd: (raw & 0x1F) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm: 0,
        imm: shift,
        sf: true,
        cond: element_size,
        size: if scalar {
            element_size
        } else if (raw >> 30) != 0 {
            16
        } else {
            8
        },
    })
}

pub(super) fn decode_simd_sli(raw: u32) -> Option<Instr> {
    let vector = (raw & 0xBF80_FC00) == 0x2F00_5400;
    let scalar = (raw & 0xFF80_FC00) == 0x7F00_5400;
    if !vector && !scalar {
        return None;
    }

    let immh = ((raw >> 19) & 0xF) as u8;
    if immh == 0 {
        return None;
    }
    let immb = ((raw >> 16) & 0x7) as u8;
    let highest = 7 - immh.leading_zeros() as u8;
    let element_size = 1u8 << highest;
    let imm = ((immh as u16) << 3) | immb as u16;
    let element_bits = element_size as u16 * 8;
    let shift = imm.checked_sub(element_bits)? as u64;

    Some(Instr {
        op: Opcode::SimdSli,
        rd: (raw & 0x1F) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm: 0,
        imm: shift,
        sf: true,
        cond: element_size,
        size: if scalar {
            element_size
        } else if (raw >> 30) != 0 {
            16
        } else {
            8
        },
    })
}
