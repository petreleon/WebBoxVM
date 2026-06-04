use super::*;

pub(super) fn decode_simd_ldst1_lane(raw: u32) -> Option<Instr> {
    let no_offset = (raw & 0xBFFF_0000) == 0x0D00_0000 || (raw & 0xBFFF_0000) == 0x0D40_0000;
    let post_index = (raw & 0xBFE0_0000) == 0x0D80_0000 || (raw & 0xBFE0_0000) == 0x0DC0_0000;
    if !no_offset && !post_index {
        return None;
    }

    let q = (raw >> 30) & 1;
    let load = ((raw >> 22) & 1) != 0;
    let rm_field = ((raw >> 16) & 0x1F) as u8;
    let opcode = (raw >> 13) & 0x7;
    let s = (raw >> 12) & 1;
    let size = (raw >> 10) & 0x3;
    let (element_size, lane) = match opcode {
        0b000 => (1, (q << 3) | (s << 2) | size),
        0b010 => {
            if (size & 1) != 0 {
                return None;
            }
            (2, (q << 2) | (s << 1) | (size >> 1))
        }
        0b100 => {
            if (size & 0b10) != 0 {
                return None;
            }
            if (size & 1) == 0 {
                (4, (q << 1) | s)
            } else {
                if s != 0 {
                    return None;
                }
                (8, q)
            }
        }
        _ => return None,
    };
    let rm = if post_index {
        if rm_field == 31 { 0xFE } else { rm_field }
    } else {
        0xFF
    };

    Some(Instr {
        op: if load {
            Opcode::SimdLd1Lane
        } else {
            Opcode::SimdSt1Lane
        },
        rd: (raw & 0x1F) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm,
        imm: lane as u64,
        sf: true,
        cond: element_size as u8,
        size: element_size as u8,
    })
}

pub(super) fn decode_simd_ld1_multi(raw: u32) -> Option<Instr> {
    let no_offset = (raw & 0xBFFF_0000) == 0x0C40_0000;
    let post_index = (raw & 0xBFE0_0000) == 0x0CC0_0000;
    if !no_offset && !post_index {
        return None;
    }

    let register_count = match (raw >> 12) & 0xF {
        0b0010 => 4,
        0b0110 => 3,
        0b0111 => 1,
        0b1010 => 2,
        _ => return None,
    };

    let q = ((raw >> 30) & 1) as u8;
    let vector_size = if q != 0 { 16 } else { 8 };
    let rm_field = ((raw >> 16) & 0x1F) as u8;
    let (rm, imm) = if post_index {
        if rm_field == 31 {
            (0xFE, register_count as u64 * vector_size as u64)
        } else {
            (rm_field, 0)
        }
    } else {
        (0xFF, 0)
    };

    Some(Instr {
        op: if register_count == 1 {
            Opcode::SimdLd1
        } else {
            Opcode::SimdLd1Multi
        },
        rd: (raw & 0x1F) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm,
        imm,
        sf: true,
        cond: register_count,
        size: vector_size,
    })
}

pub(super) fn decode_simd_st1_multi(raw: u32) -> Option<Instr> {
    let no_offset = (raw & 0xBFFF_0000) == 0x0C00_0000;
    let post_index = (raw & 0xBFE0_0000) == 0x0C80_0000;
    if !no_offset && !post_index {
        return None;
    }

    let register_count = match (raw >> 12) & 0xF {
        0b0010 => 4,
        0b0110 => 3,
        0b0111 => 1,
        0b1010 => 2,
        _ => return None,
    };

    let q = ((raw >> 30) & 1) as u8;
    let vector_size = if q != 0 { 16 } else { 8 };
    let rm_field = ((raw >> 16) & 0x1F) as u8;
    let (rm, imm) = if post_index {
        if rm_field == 31 {
            (0xFE, register_count as u64 * vector_size as u64)
        } else {
            (rm_field, 0)
        }
    } else {
        (0xFF, 0)
    };

    Some(Instr {
        op: Opcode::SimdSt1Multi,
        rd: (raw & 0x1F) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm,
        imm,
        sf: true,
        cond: register_count,
        size: vector_size,
    })
}
