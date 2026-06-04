use super::*;

#[test]
fn decode_simd_fp_minmax_vector_forms_cross_checked_with_disarm64() {
    let cases = [
        (0x4E22_F420, Opcode::SimdFpFmaxVec, "fmax"),
        (0x4EE5_F483, Opcode::SimdFpFminVec, "fmin"),
        (0x4E28_C4E6, Opcode::SimdFpFmaxnmVec, "fmaxnm"),
        (0x4EEB_C549, Opcode::SimdFpFminnmVec, "fminnm"),
    ];
    assert_decode_cases(&cases);

    let fmax = decode(0x4E22_F420).unwrap();
    assert_eq!((fmax.rd, fmax.rn, fmax.rm), (0, 1, 2));
    assert_eq!((fmax.imm, fmax.size), (4, 16));
    let fmin = decode(0x4EE5_F483).unwrap();
    assert_eq!((fmin.rd, fmin.rn, fmin.rm), (3, 4, 5));
    assert_eq!((fmin.imm, fmin.size), (8, 16));

    for raw in [0x0E62_F420, 0x2E62_F420, 0x0E62_C420, 0x2E62_C420] {
        assert!(decode(raw).is_none(), "raw=0x{raw:08x}");
    }
}
