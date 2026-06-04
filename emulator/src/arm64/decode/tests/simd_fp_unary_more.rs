use super::*;

#[test]
fn decode_simd_fp_abs_and_round_vectors() {
    let cases = [
        (0x4EA0_F81F, Opcode::SimdFpAbsVec, "fabs"),
        (0x4EE0_F81D, Opcode::SimdFpAbsVec, "fabs"),
        (0x6E21_8B9C, Opcode::SimdFpFrintaVec, "frinta"),
        (0x6E61_8B9C, Opcode::SimdFpFrintaVec, "frinta"),
    ];
    assert_decode_cases(&cases);

    let fabs = decode(0x4EA0_F81F).unwrap();
    assert_eq!(fabs.rd, 31);
    assert_eq!(fabs.rn, 0);
    assert_eq!(fabs.imm, 4);
    assert_eq!(fabs.size, 16);

    let fabs_double = decode(0x4EE0_F81D).unwrap();
    assert_eq!(fabs_double.imm, 8);
    assert_eq!(fabs_double.size, 16);

    let frinta = decode(0x2E21_8B9C).unwrap();
    assert_eq!(frinta.rd, 28);
    assert_eq!(frinta.rn, 28);
    assert_eq!(frinta.imm, 4);
    assert_eq!(frinta.size, 8);
    assert!(decode(0x2E61_8B9C).is_none());
}
