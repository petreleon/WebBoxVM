pub(super) fn simd_ld1_multi_register_count(raw: u32) -> Option<u8> {
    let no_offset = (raw & 0xBFFF_0000) == 0x0C40_0000;
    let post_index = (raw & 0xBFE0_0000) == 0x0CC0_0000;
    if !no_offset && !post_index {
        return None;
    }

    match (raw >> 12) & 0xF {
        0b0010 => Some(4),
        0b0110 => Some(3),
        0b0111 => Some(1),
        0b1010 => Some(2),
        _ => None,
    }
}

pub(super) fn simd_smov_is_valid(raw: u32) -> bool {
    if (raw & 0xBFE0_FC00) != 0x0E00_2C00 {
        return false;
    }

    let q = ((raw >> 30) & 1) != 0;
    let imm5 = ((raw >> 16) & 0x1F) as u8;
    let Some((element_size, _)) = simd_move_element(imm5) else {
        return false;
    };
    let data_size = if q { 8 } else { 4 };
    (element_size as usize) < data_size
}

pub(super) fn simd_move_element(imm5: u8) -> Option<(u8, u8)> {
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

pub(super) fn simd_ldst1_single_lane(raw: u32) -> bool {
    let no_offset = (raw & 0xBFFF_0000) == 0x0D00_0000 || (raw & 0xBFFF_0000) == 0x0D40_0000;
    let post_index = (raw & 0xBFE0_0000) == 0x0D80_0000 || (raw & 0xBFE0_0000) == 0x0DC0_0000;
    if !no_offset && !post_index {
        return false;
    }

    matches!((raw >> 13) & 0x7, 0b000 | 0b010 | 0b100)
}

pub(super) fn simd_ld_structure_elements(raw: u32) -> Option<u8> {
    let no_offset = (raw & 0xBFFF_0000) == 0x0C40_0000;
    let post_index = (raw & 0xBFE0_0000) == 0x0CC0_0000;
    if !no_offset && !post_index {
        return None;
    }

    let q = ((raw >> 30) & 1) as u8;
    let size = ((raw >> 10) & 0x3) as u8;
    if size == 3 && q == 0 {
        return None;
    }

    match (raw >> 12) & 0xF {
        0b0000 => Some(4),
        0b0100 => Some(3),
        0b1000 => Some(2),
        _ => None,
    }
}

pub(super) fn simd_st1_multi_register_count(raw: u32) -> Option<u8> {
    let no_offset = (raw & 0xBFFF_0000) == 0x0C00_0000;
    let post_index = (raw & 0xBFE0_0000) == 0x0C80_0000;
    if !no_offset && !post_index {
        return None;
    }

    match (raw >> 12) & 0xF {
        0b0010 => Some(4),
        0b0110 => Some(3),
        0b0111 => Some(1),
        0b1010 => Some(2),
        _ => None,
    }
}

pub(super) fn simd_fmulx_elem(raw: u32) -> bool {
    (raw & 0xBF80_F400) == 0x2F80_9000 || (raw & 0xBF80_F400) == 0x3F80_9000
}

pub(super) fn simd_fmulx_direct(raw: u32) -> bool {
    (raw & 0xBFA0_FC00) == 0x0E20_DC00 || (raw & 0xFF20_FC00) == 0x5E20_DC00
}

pub(super) fn simd_minmax(raw: u32, base: u32) -> bool {
    (raw & 0xBF20_FC00) == base && ((raw >> 22) & 0x3) != 0x3
}

pub(super) fn simd_across_minmax(raw: u32, base: u32) -> bool {
    let q = ((raw >> 30) & 1) != 0;
    let element_size = 1u64 << ((raw >> 22) & 0x3);
    (raw & 0xBF3F_FC00) == base && element_size < 8 && (element_size != 4 || q)
}

pub(super) fn simd_fp_reduce_s(raw: u32, base: u32) -> bool {
    (raw & 0xFFFF_FC00) == base
}

pub(super) fn simd_fp_binary(raw: u32, base: u32) -> bool {
    (raw & 0xBFA0_FC00) == base && (((raw >> 22) & 1) == 0 || ((raw >> 30) & 1) != 0)
}

pub(super) fn simd_fp_pairwise_scalar(raw: u32, base: u32) -> bool {
    (raw & 0xFFBF_FC00) == base
}

pub(super) fn simd_fp_compare(raw: u32, vector_base: u32, scalar_base: u32) -> bool {
    (raw & 0xBFA0_FC00) == vector_base || (raw & 0xFFA0_FC00) == scalar_base
}

pub(super) fn simd_fp_zero_compare(raw: u32, vector_base: u32, scalar_base: u32) -> bool {
    (raw & 0xBFBF_FC00) == vector_base || (raw & 0xFFBF_FC00) == scalar_base
}

pub(super) fn sve_fp_size_valid(raw: u32) -> bool {
    ((raw >> 22) & 0x3) != 0
}

pub(super) fn sve_fp_cmp(raw: u32, vec_base: u32, zero_base: u32) -> bool {
    (raw & 0xFF20_E010) == vec_base || (raw & 0xFF3F_E010) == zero_base
}
