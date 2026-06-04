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

#[test]
fn decode_simd_fp_mulx_by_element_forms_cross_checked_with_disarm64() {
    let cases = [
        (0x2F82_9020, Opcode::SimdFpMulxElem, "fmulx"),
        (0x6FA5_9883, Opcode::SimdFpMulxElem, "fmulx"),
        (0x6FC8_98E6, Opcode::SimdFpMulxElem, "fmulx"),
        (0x7F8B_9949, Opcode::SimdFpMulxElem, "fmulx"),
        (0x7FCE_91AC, Opcode::SimdFpMulxElem, "fmulx"),
        (0x7FD1_9A0F, Opcode::SimdFpMulxElem, "fmulx"),
    ];
    assert_decode_cases(&cases);

    let v2s = decode(0x2F82_9020).unwrap();
    assert_eq!((v2s.rd, v2s.rn, v2s.rm, v2s.cond), (0, 1, 2, 0));
    assert_eq!((v2s.imm, v2s.size), (4, 8));

    let v4s = decode(0x6FA5_9883).unwrap();
    assert_eq!((v4s.rd, v4s.rn, v4s.rm, v4s.cond), (3, 4, 5, 3));
    assert_eq!((v4s.imm, v4s.size), (4, 16));

    let v2d = decode(0x6FC8_98E6).unwrap();
    assert_eq!((v2d.rd, v2d.rn, v2d.rm, v2d.cond), (6, 7, 8, 1));
    assert_eq!((v2d.imm, v2d.size), (8, 16));

    let scalar_s = decode(0x7F8B_9949).unwrap();
    assert_eq!(
        (scalar_s.rd, scalar_s.rn, scalar_s.rm, scalar_s.cond),
        (9, 10, 11, 2)
    );
    assert_eq!((scalar_s.imm, scalar_s.size), (4, 4));

    let scalar_d = decode(0x7FD1_9A0F).unwrap();
    assert_eq!(
        (scalar_d.rd, scalar_d.rn, scalar_d.rm, scalar_d.cond),
        (15, 16, 17, 1)
    );
    assert_eq!((scalar_d.imm, scalar_d.size), (8, 8));

    assert!(decode(0x2FC8_98E6).is_none());
    assert!(decode(0x7FEE_91AC).is_none());
}
