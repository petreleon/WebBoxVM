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

    let fcmle = decode(0x6EE0_DB7D).unwrap();
    assert_eq!((fcmle.rd, fcmle.rn, fcmle.rm), (29, 27, 0));
    assert_eq!((fcmle.imm, fcmle.size), (8, 16));

    assert!(decode(0x2E60_E400).is_none());
}
