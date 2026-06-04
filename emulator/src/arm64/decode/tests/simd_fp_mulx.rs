use super::*;

#[test]
fn decode_simd_fp_mulx_forms_cross_checked_with_disarm64() {
    let cases = [
        (0x5E22_DC20, Opcode::SimdFpMulx, "fmulx"),
        (0x5E65_DC83, Opcode::SimdFpMulx, "fmulx"),
        (0x0E28_DCE6, Opcode::SimdFpMulx, "fmulx"),
        (0x4E2B_DD49, Opcode::SimdFpMulx, "fmulx"),
        (0x4E6E_DDAC, Opcode::SimdFpMulx, "fmulx"),
    ];
    assert_decode_cases(&cases);

    let scalar_s = decode(0x5E22_DC20).unwrap();
    assert_eq!((scalar_s.rd, scalar_s.rn, scalar_s.rm), (0, 1, 2));
    assert_eq!((scalar_s.imm, scalar_s.size), (4, 4));

    let scalar_d = decode(0x5E65_DC83).unwrap();
    assert_eq!((scalar_d.rd, scalar_d.rn, scalar_d.rm), (3, 4, 5));
    assert_eq!((scalar_d.imm, scalar_d.size), (8, 8));

    let v2s = decode(0x0E28_DCE6).unwrap();
    assert_eq!((v2s.rd, v2s.rn, v2s.rm), (6, 7, 8));
    assert_eq!((v2s.imm, v2s.size), (4, 8));

    let v4s = decode(0x4E2B_DD49).unwrap();
    assert_eq!((v4s.rd, v4s.rn, v4s.rm), (9, 10, 11));
    assert_eq!((v4s.imm, v4s.size), (4, 16));

    let v2d = decode(0x4E6E_DDAC).unwrap();
    assert_eq!((v2d.rd, v2d.rn, v2d.rm), (12, 13, 14));
    assert_eq!((v2d.imm, v2d.size), (8, 16));

    assert!(decode(0x0E60_DC00).is_none());
}
