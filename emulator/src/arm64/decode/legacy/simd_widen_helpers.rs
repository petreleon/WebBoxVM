use super::*;

pub(super) fn decode_simd_ushll(raw: u32) -> Option<Instr> {
    if (raw & 0xBF80_FC00) != 0x2F00_A400 {
        return None;
    }

    let immh = ((raw >> 19) & 0xF) as u8;
    if immh == 0 {
        return None;
    }
    let immb = ((raw >> 16) & 0x7) as u8;
    let highest = 7 - immh.leading_zeros() as u8;
    let element_size = 1u8 << highest;
    if element_size > 4 {
        return None;
    }
    let imm = ((immh as u16) << 3) | immb as u16;
    let element_bits = element_size as u16 * 8;
    let shift = imm.checked_sub(element_bits)? as u64;

    Some(Instr {
        op: Opcode::SimdUshll,
        rd: (raw & 0x1F) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm: 0,
        imm: shift,
        sf: true,
        cond: element_size,
        size: 16,
    })
}

pub(super) fn decode_simd_sshll(raw: u32) -> Option<Instr> {
    if (raw & 0xBF80_FC00) != 0x0F00_A400 {
        return None;
    }

    let immh = ((raw >> 19) & 0xF) as u8;
    if immh == 0 {
        return None;
    }
    let immb = ((raw >> 16) & 0x7) as u8;
    let highest = 7 - immh.leading_zeros() as u8;
    let element_size = 1u8 << highest;
    if element_size > 4 {
        return None;
    }
    let imm = ((immh as u16) << 3) | immb as u16;
    let element_bits = element_size as u16 * 8;
    let shift = imm.checked_sub(element_bits)? as u64;

    Some(Instr {
        op: Opcode::SimdSshll,
        rd: (raw & 0x1F) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm: 0,
        imm: shift,
        sf: true,
        cond: element_size,
        size: 16,
    })
}

pub(super) fn decode_simd_widen_mul_by_element(raw: u32) -> Option<Instr> {
    let op = match raw & 0xBF00_F400 {
        0x2F00_2000 => Opcode::SimdUmlal,
        0x2F00_A000 => Opcode::SimdUmullElem,
        _ => return None,
    };
    let size = ((raw >> 22) & 0x3) as u8;
    let q = ((raw >> 30) & 1) != 0;
    let l = ((raw >> 21) & 1) as u8;
    let m_bit = ((raw >> 20) & 1) as u8;
    let rm_low = ((raw >> 16) & 0xF) as u8;
    let h = ((raw >> 11) & 1) as u8;
    let (element_size, rm, index) = match size {
        0b01 => (2, rm_low, (h << 2) | (l << 1) | m_bit),
        0b10 => (4, (m_bit << 4) | rm_low, (h << 1) | l),
        _ => return None,
    };

    Some(Instr {
        op,
        rd: (raw & 0x1F) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm,
        imm: index as u64,
        sf: q,
        cond: element_size,
        size: 16,
    })
}

pub(super) fn decode_simd_widen_mul_vector(raw: u32) -> Option<Instr> {
    let op = match raw & 0xBF20_FC00 {
        0x2E20_8000 => Opcode::SimdUmlalVec,
        0x2E20_C000 => Opcode::SimdUmull,
        _ => return None,
    };
    let element_size = 1u8 << ((raw >> 22) & 0x3);
    if element_size > 4 {
        return None;
    }

    Some(Instr {
        op,
        rd: (raw & 0x1F) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm: ((raw >> 16) & 0x1F) as u8,
        imm: 0,
        sf: ((raw >> 30) & 1) != 0,
        cond: element_size,
        size: 16,
    })
}
