use super::*;

pub(super) fn decode_simd_ld_structure_multi(raw: u32) -> Option<Instr> {
    decode_simd_structure_multi(raw, true)
}

pub(super) fn decode_simd_st_structure_multi(raw: u32) -> Option<Instr> {
    decode_simd_structure_multi(raw, false)
}

fn decode_simd_structure_multi(raw: u32, load: bool) -> Option<Instr> {
    let no_offset_base = if load { 0x0C40_0000 } else { 0x0C00_0000 };
    let post_index_base = if load { 0x0CC0_0000 } else { 0x0C80_0000 };
    let no_offset = (raw & 0xBFFF_0000) == no_offset_base;
    let post_index = (raw & 0xBFE0_0000) == post_index_base;
    if !no_offset && !post_index {
        return None;
    }

    let q = ((raw >> 30) & 1) as u8;
    let size = ((raw >> 10) & 0x3) as u8;
    if size == 3 && q == 0 {
        return None;
    }
    let (load_op, store_op, structure_count) = match (raw >> 12) & 0xF {
        0b0000 => (Opcode::SimdLd4, Opcode::SimdSt4, 4),
        0b0100 => (Opcode::SimdLd3, Opcode::SimdSt3, 3),
        0b1000 => (Opcode::SimdLd2, Opcode::SimdSt2, 2),
        _ => return None,
    };

    let vector_size = if q != 0 { 16 } else { 8 };
    let rm_field = ((raw >> 16) & 0x1F) as u8;
    let (rm, imm) = if post_index {
        if rm_field == 31 {
            (0xFE, structure_count as u64 * vector_size as u64)
        } else {
            (rm_field, 0)
        }
    } else {
        (0xFF, 0)
    };

    Some(Instr {
        op: if load { load_op } else { store_op },
        rd: (raw & 0x1F) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm,
        imm,
        sf: true,
        cond: size,
        size: vector_size,
    })
}

pub(super) fn decode_movi_doubleword_imm(imm8: u32) -> u64 {
    let mut value = 0u64;
    for byte in 0..8 {
        if ((imm8 >> byte) & 1) != 0 {
            value |= 0xffu64 << (byte * 8);
        }
    }
    value
}
