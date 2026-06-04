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
