use super::*;

pub(super) fn decode_simd_int_fp_convert(
    raw: u32,
    scalar_pattern: u32,
    vector_pattern: u32,
    op: Opcode,
) -> Option<Instr> {
    let rd = (raw & 0x1F) as u8;
    let rn = ((raw >> 5) & 0x1F) as u8;
    let element_size = if ((raw >> 22) & 1) != 0 { 8 } else { 4 };

    if (raw & 0xFFBF_FC00) == scalar_pattern {
        return Some(Instr {
            op,
            rd,
            rn,
            rm: 0,
            imm: element_size as u64,
            sf: true,
            cond: 0,
            size: element_size,
        });
    }

    if (raw & 0xBFBF_FC00) == vector_pattern {
        let q = ((raw >> 30) & 1) != 0;
        if element_size == 8 && !q {
            return None;
        }
        return Some(Instr {
            op,
            rd,
            rn,
            rm: 0,
            imm: element_size as u64,
            sf: true,
            cond: 0,
            size: if q { 16 } else { 8 },
        });
    }

    None
}

pub(super) fn decode_simd_fp_to_int_vector(raw: u32, pattern: u32, op: Opcode) -> Option<Instr> {
    if (raw & 0xBFBF_FC00) != pattern {
        return None;
    }
    let q = ((raw >> 30) & 1) != 0;
    let element_size = if ((raw >> 22) & 1) != 0 { 8 } else { 4 };
    if element_size == 8 && !q {
        return None;
    }
    Some(Instr {
        op,
        rd: (raw & 0x1F) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm: 0,
        imm: element_size as u64,
        sf: true,
        cond: 0,
        size: if q { 16 } else { 8 },
    })
}

pub(super) fn decode_umov_element(imm5: u8) -> Option<(u8, u8)> {
    if imm5 & 0b00001 != 0 {
        Some((1, imm5 >> 1))
    } else if imm5 & 0b00010 != 0 {
        Some((2, imm5 >> 2))
    } else if imm5 & 0b00100 != 0 {
        Some((4, imm5 >> 3))
    } else if imm5 & 0b01000 != 0 {
        Some((8, imm5 >> 4))
    } else {
        None
    }
}

pub(super) fn decode_simd_fp_binary(raw: u32, pattern: u32, op: Opcode) -> Option<Instr> {
    if (raw & 0xBFA0_FC00) != pattern {
        return None;
    }
    let q = ((raw >> 30) & 1) != 0;
    let element_size = if ((raw >> 22) & 1) != 0 { 8 } else { 4 };
    if element_size == 8 && !q {
        return None;
    }
    Some(Instr {
        op,
        rd: (raw & 0x1F) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm: ((raw >> 16) & 0x1F) as u8,
        imm: element_size,
        sf: true,
        cond: 0,
        size: if q { 16 } else { 8 },
    })
}
