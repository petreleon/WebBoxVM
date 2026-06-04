use super::*;

#[test]
fn decode_simd_fcmlt_zero_vector_forms() {
    assert_decode_cases(&[
        (0x4EA0_E803, Opcode::SimdFpFcmltZero, "fcmlt"),
        (0x4EE0_EBC6, Opcode::SimdFpFcmltZero, "fcmlt"),
    ]);

    let singles = decode(0x4EA0_E803).unwrap();
    assert_eq!(singles.rd, 3);
    assert_eq!(singles.rn, 0);
    assert_eq!(singles.imm, 4);
    assert_eq!(singles.size, 16);

    let doubles = decode(0x4EE0_EBC6).unwrap();
    assert_eq!(doubles.rd, 6);
    assert_eq!(doubles.rn, 30);
    assert_eq!(doubles.imm, 8);
    assert_eq!(doubles.size, 16);

    assert!(decode(0x0EE0_E800).is_none());
}

#[test]
fn decode_simd_fp_compare_forms_cross_checked_with_disarm64() {
    assert_decode_cases(&[
        (0x0E22_E420, Opcode::SimdFpFcmeqVec, "fcmeq"),
        (0x4E25_E483, Opcode::SimdFpFcmeqVec, "fcmeq"),
        (0x4E68_E4E6, Opcode::SimdFpFcmeqVec, "fcmeq"),
        (0x6E20_E7FE, Opcode::SimdFpFcmgeVec, "fcmge"),
        (0x6E63_E7C3, Opcode::SimdFpFcmgeVec, "fcmge"),
        (0x6EBC_E77C, Opcode::SimdFpFcmgtVec, "fcmgt"),
        (0x6EFB_E4FB, Opcode::SimdFpFcmgtVec, "fcmgt"),
        (0x4EE0_DBA0, Opcode::SimdFpFcmeqZero, "fcmeq"),
        (0x4EE0_DBBF, Opcode::SimdFpFcmeqZero, "fcmeq"),
        (0x6EA0_D83C, Opcode::SimdFpFcmleZero, "fcmle"),
        (0x6EE0_DB7D, Opcode::SimdFpFcmleZero, "fcmle"),
    ]);

    let fcmge = decode(0x6E20_E7FE).unwrap();
    assert_eq!((fcmge.rd, fcmge.rn, fcmge.rm), (30, 31, 0));
    assert_eq!((fcmge.imm, fcmge.size), (4, 16));
    let fcmeq = decode(0x4E68_E4E6).unwrap();
    assert_eq!((fcmeq.rd, fcmeq.rn, fcmeq.rm), (6, 7, 8));
    assert_eq!((fcmeq.imm, fcmeq.size), (8, 16));

    let fcmle = decode(0x6EE0_DB7D).unwrap();
    assert_eq!((fcmle.rd, fcmle.rn, fcmle.rm), (29, 27, 0));
    assert_eq!((fcmle.imm, fcmle.size), (8, 16));

    assert!(decode(0x2E60_E400).is_none());
}

#[test]
fn decode_simd_fp_scalar_compare_forms_cross_checked_with_disarm64() {
    assert_decode_cases(&[
        (0x5E2B_E549, Opcode::SimdFpFcmeqVec, "fcmeq"),
        (0x5E6E_E5AC, Opcode::SimdFpFcmeqVec, "fcmeq"),
        (0x7E22_E420, Opcode::SimdFpFcmgeVec, "fcmge"),
        (0x7EA5_E483, Opcode::SimdFpFcmgtVec, "fcmgt"),
        (0x7E28_ECE6, Opcode::SimdFpFacgeVec, "facge"),
        (0x7EAB_ED49, Opcode::SimdFpFacgtVec, "facgt"),
        (0x7E74_E672, Opcode::SimdFpFcmgeVec, "fcmge"),
        (0x7EF7_E6D5, Opcode::SimdFpFcmgtVec, "fcmgt"),
        (0x7E7A_EF38, Opcode::SimdFpFacgeVec, "facge"),
        (0x7EFD_EF9B, Opcode::SimdFpFacgtVec, "facgt"),
    ]);

    let fcmeq = decode(0x5E2B_E549).unwrap();
    assert_eq!((fcmeq.rd, fcmeq.rn, fcmeq.rm), (9, 10, 11));
    assert_eq!((fcmeq.imm, fcmeq.size), (4, 4));
    let fcmge = decode(0x7E22_E420).unwrap();
    assert_eq!((fcmge.rd, fcmge.rn, fcmge.rm), (0, 1, 2));
    assert_eq!((fcmge.imm, fcmge.size), (4, 4));
    let facgt = decode(0x7EFD_EF9B).unwrap();
    assert_eq!((facgt.rd, facgt.rn, facgt.rm), (27, 28, 29));
    assert_eq!((facgt.imm, facgt.size), (8, 8));
}

#[test]
fn decode_simd_fp_zero_compare_ge_gt_and_scalar_forms() {
    assert_decode_cases(&[
        (0x2EA0_C820, Opcode::SimdFpFcmgeZero, "fcmge"),
        (0x4EA0_C862, Opcode::SimdFpFcmgtZero, "fcmgt"),
        (0x6EE0_C8A4, Opcode::SimdFpFcmgeZero, "fcmge"),
        (0x4EE0_C8E6, Opcode::SimdFpFcmgtZero, "fcmgt"),
        (0x7EA0_C820, Opcode::SimdFpFcmgeZero, "fcmge"),
        (0x5EA0_C862, Opcode::SimdFpFcmgtZero, "fcmgt"),
        (0x5EA0_D9AC, Opcode::SimdFpFcmeqZero, "fcmeq"),
        (0x7EA0_D9EE, Opcode::SimdFpFcmleZero, "fcmle"),
        (0x5EA0_EA30, Opcode::SimdFpFcmltZero, "fcmlt"),
        (0x7EE0_C8A4, Opcode::SimdFpFcmgeZero, "fcmge"),
        (0x5EE0_C8E6, Opcode::SimdFpFcmgtZero, "fcmgt"),
        (0x5EE0_DBFE, Opcode::SimdFpFcmeqZero, "fcmeq"),
        (0x7EE0_D820, Opcode::SimdFpFcmleZero, "fcmle"),
        (0x5EE0_E862, Opcode::SimdFpFcmltZero, "fcmlt"),
    ]);

    let vector = decode(0x4EA0_C862).unwrap();
    assert_eq!((vector.rd, vector.rn, vector.rm), (2, 3, 0));
    assert_eq!((vector.imm, vector.size), (4, 16));
    let scalar = decode(0x5EE0_E862).unwrap();
    assert_eq!((scalar.rd, scalar.rn, scalar.rm), (2, 3, 0));
    assert_eq!((scalar.imm, scalar.size), (8, 8));
}
