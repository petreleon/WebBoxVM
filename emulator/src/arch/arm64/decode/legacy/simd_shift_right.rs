use super::*;

pub(super) fn decode_simd_ushr(raw: u32) -> Option<Instr> {
    let vector = (raw & 0xBF80_FC00) == 0x2F00_0400;
    let scalar = (raw & 0xFF80_FC00) == 0x7F00_0400;
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
    if vector && element_size == 8 && ((raw >> 30) & 1) == 0 {
        return None;
    }
    let imm = ((immh as u16) << 3) | immb as u16;
    let element_bits = element_size as u16 * 8;
    let shift = (element_bits * 2).checked_sub(imm)? as u64;

    Some(Instr {
        op: Opcode::SimdUshr,
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

pub(super) fn decode_simd_usra(raw: u32) -> Option<Instr> {
    if (raw & 0xBF80_FC00) != 0x2F00_1400 {
        return None;
    }

    let immh = ((raw >> 19) & 0xF) as u8;
    if immh == 0 {
        return None;
    }
    let immb = ((raw >> 16) & 0x7) as u8;
    let highest = 7 - immh.leading_zeros() as u8;
    let element_size = 1u8 << highest;
    if element_size == 8 && ((raw >> 30) & 1) == 0 {
        return None;
    }
    let imm = ((immh as u16) << 3) | immb as u16;
    let element_bits = element_size as u16 * 8;
    let shift = (element_bits * 2).checked_sub(imm)? as u64;

    Some(Instr {
        op: Opcode::SimdUsra,
        rd: (raw & 0x1F) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm: 0,
        imm: shift,
        sf: true,
        cond: element_size,
        size: if (raw >> 30) != 0 { 16 } else { 8 },
    })
}

pub(super) fn decode_simd_sshr(raw: u32) -> Option<Instr> {
    if (raw & 0xFF80_FC00) == 0x5F00_0400 {
        let immh = ((raw >> 19) & 0xF) as u8;
        if (immh & 0x8) == 0 {
            return None;
        }
        let immb = ((raw >> 16) & 0x7) as u8;
        let imm = ((immh as u16) << 3) | immb as u16;
        let shift = 128u16.checked_sub(imm)? as u64;
        return Some(Instr {
            op: Opcode::SimdSshr,
            rd: (raw & 0x1F) as u8,
            rn: ((raw >> 5) & 0x1F) as u8,
            rm: 0,
            imm: shift,
            sf: true,
            cond: 8,
            size: 8,
        });
    }

    if (raw & 0xBF80_FC00) != 0x0F00_0400 {
        return None;
    }

    let immh = ((raw >> 19) & 0xF) as u8;
    if immh == 0 {
        return None;
    }
    let immb = ((raw >> 16) & 0x7) as u8;
    let highest = 7 - immh.leading_zeros() as u8;
    let element_size = 1u8 << highest;
    if element_size == 8 && ((raw >> 30) & 1) == 0 {
        return None;
    }
    let imm = ((immh as u16) << 3) | immb as u16;
    let element_bits = element_size as u16 * 8;
    let shift = (element_bits * 2).checked_sub(imm)? as u64;

    Some(Instr {
        op: Opcode::SimdSshr,
        rd: (raw & 0x1F) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm: 0,
        imm: shift,
        sf: true,
        cond: element_size,
        size: if (raw >> 30) != 0 { 16 } else { 8 },
    })
}
