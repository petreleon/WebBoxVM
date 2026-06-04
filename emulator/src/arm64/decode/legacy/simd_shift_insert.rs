use super::*;

pub(super) fn decode_simd_sri(raw: u32) -> Option<Instr> {
    let vector = (raw & 0xBF80_FC00) == 0x2F00_4400;
    let scalar = (raw & 0xFF80_FC00) == 0x7F00_4400;
    if !vector && !scalar {
        return None;
    }

    let immh = ((raw >> 19) & 0xF) as u8;
    if immh == 0 {
        return None;
    }
    let q = ((raw >> 30) & 1) != 0;
    if vector && (immh & 0b1000) != 0 && !q {
        return None;
    }
    if scalar && (immh & 0b1000) == 0 {
        return None;
    }
    let immb = ((raw >> 16) & 0x7) as u8;
    let highest = 7 - immh.leading_zeros() as u8;
    let element_size = 1u8 << highest;
    let imm = ((immh as u16) << 3) | immb as u16;
    let element_bits = element_size as u16 * 8;
    let shift = (element_bits * 2).checked_sub(imm)? as u64;

    Some(Instr {
        op: Opcode::SimdSri,
        rd: (raw & 0x1F) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm: 0,
        imm: shift,
        sf: true,
        cond: element_size,
        size: if scalar {
            element_size
        } else if q {
            16
        } else {
            8
        },
    })
}

pub(super) fn decode_simd_shrn(raw: u32) -> Option<Instr> {
    let q = ((raw >> 30) & 1) != 0;
    let op = match raw & 0xBF80_FC00 {
        0x0F00_8400 => {
            if q {
                Opcode::SimdShrn2
            } else {
                Opcode::SimdShrn
            }
        }
        0x0F00_8C00 => {
            if q {
                Opcode::SimdRshrn2
            } else {
                Opcode::SimdRshrn
            }
        }
        _ => return None,
    };
    let immh = ((raw >> 19) & 0xF) as u8;
    if immh == 0 || (immh & 0b1000) != 0 {
        return None;
    }
    let immb = ((raw >> 16) & 0x7) as u8;
    let highest = 7 - immh.leading_zeros() as u8;
    let dest_element_size = 1u8 << highest;
    let imm = ((immh as u16) << 3) | immb as u16;
    let dest_element_bits = dest_element_size as u16 * 8;
    let shift = (dest_element_bits * 2).checked_sub(imm)? as u64;

    Some(Instr {
        op,
        rd: (raw & 0x1F) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm: 0,
        imm: shift,
        sf: false,
        cond: dest_element_size,
        size: if q { 16 } else { 8 },
    })
}
