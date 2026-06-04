use super::*;

pub(super) fn map(raw: u32, m: disarm64::decoder::Mnemonic) -> Option<Opcode> {
    use disarm64::decoder::Mnemonic as M;
    Some(match m {
        M::r#ld1r => Opcode::SimdLd1r,
        M::r#ld1 if simd_ldst1_single_lane(raw) => Opcode::SimdLd1Lane,
        M::r#ld1 if simd_ld1_multi_register_count(raw) == Some(1) => Opcode::SimdLd1,
        M::r#ld1 if simd_ld1_multi_register_count(raw).is_some() => Opcode::SimdLd1Multi,
        M::r#st1 if simd_ldst1_single_lane(raw) => Opcode::SimdSt1Lane,
        M::r#ld4 if simd_ld1_multi_register_count(raw) == Some(1) => Opcode::SimdLd1,
        M::r#ld4 if simd_ld1_multi_register_count(raw).is_some() => Opcode::SimdLd1Multi,
        M::r#ld2 if simd_ld_structure_elements(raw) == Some(2) => Opcode::SimdLd2,
        M::r#ld3 if simd_ld_structure_elements(raw) == Some(3) => Opcode::SimdLd3,
        M::r#ld4 if simd_ld_structure_elements(raw) == Some(2) => Opcode::SimdLd2,
        M::r#ld4 if simd_ld_structure_elements(raw) == Some(3) => Opcode::SimdLd3,
        M::r#ld4 if simd_ld_structure_elements(raw) == Some(4) => Opcode::SimdLd4,
        M::r#st1 if simd_st1_multi_register_count(raw).is_some() => Opcode::SimdSt1Multi,
        M::r#st4 if simd_st1_multi_register_count(raw).is_some() => Opcode::SimdSt1Multi,
        M::r#st4 if (raw & 0xFFFF_FC00) == 0x4C9F_7800 => Opcode::SimdSt4Single,
        _ => return None,
    })
}
