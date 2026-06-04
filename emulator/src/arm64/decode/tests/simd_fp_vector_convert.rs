use super::*;

#[test]
fn decode_simd_fcvtas_vector_forms() {
    assert_decode_cases(&[
        (0x4E21_C802, Opcode::SimdFcvtas, "fcvtas"),
        (0x4E61_CBBB, Opcode::SimdFcvtas, "fcvtas"),
    ]);

    let singles = decode(0x4E21_C802).unwrap();
    assert_eq!(singles.rd, 2);
    assert_eq!(singles.rn, 0);
    assert_eq!(singles.imm, 4);
    assert_eq!(singles.size, 16);

    let doubles = decode(0x4E61_CBBB).unwrap();
    assert_eq!(doubles.rd, 27);
    assert_eq!(doubles.rn, 29);
    assert_eq!(doubles.imm, 8);
    assert_eq!(doubles.size, 16);

    assert!(decode(0x0E61_C800).is_none());
}
