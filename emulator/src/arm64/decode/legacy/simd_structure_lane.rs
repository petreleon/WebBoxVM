use super::*;

pub(super) fn decode_simd_st4_single_lane(raw: u32) -> Option<Instr> {
    decode_simd_ldst4_single_lane(raw, false)
}

pub(super) fn decode_simd_ld4_single_lane(raw: u32) -> Option<Instr> {
    decode_simd_ldst4_single_lane(raw, true)
}

fn decode_simd_ldst4_single_lane(raw: u32, load: bool) -> Option<Instr> {
    let no_offset_base = if load { 0x0D60_2000 } else { 0x0D20_0000 };
    let post_base = if load { 0x0DE0_2000 } else { 0x0DA0_0000 };
    let no_offset_mask = if load { 0xBFFF_2000 } else { 0xBFFF_0000 };
    let post_mask = if load { 0xBFE0_2000 } else { 0xBFE0_0000 };
    let no_offset = (raw & no_offset_mask) == no_offset_base;
    let post_index = (raw & post_mask) == post_base;
    if !no_offset && !post_index {
        return None;
    }

    let q = (raw >> 30) & 1;
    let s = (raw >> 12) & 1;
    let size = (raw >> 10) & 0x3;
    let (element_size, lane) = match (raw >> 13) & 0x7 {
        0b001 => (1, (q << 3) | (s << 2) | size),
        0b011 if (size & 1) == 0 => (2, (q << 2) | (s << 1) | (size >> 1)),
        0b101 if (size & 0b10) == 0 && (size & 1) == 0 => (4, (q << 1) | s),
        0b101 if (size & 0b10) == 0 && s == 0 => (8, q),
        _ => return None,
    };

    let rm_field = ((raw >> 16) & 0x1F) as u8;
    let rm = if post_index {
        if rm_field == 31 {
            0xFE
        } else {
            rm_field
        }
    } else {
        0xFF
    };

    Some(Instr {
        op: if load {
            Opcode::SimdLd4Single
        } else {
            Opcode::SimdSt4Single
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
