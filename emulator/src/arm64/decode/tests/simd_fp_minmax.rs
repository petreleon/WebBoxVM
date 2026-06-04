use super::*;

#[test]
fn decode_simd_fp_minmax_vector_forms_cross_checked_with_disarm64() {
    let cases = [
        (0x4E22_F420, Opcode::SimdFpFmaxVec, "fmax"),
        (0x4EE5_F483, Opcode::SimdFpFminVec, "fmin"),
        (0x4E28_C4E6, Opcode::SimdFpFmaxnmVec, "fmaxnm"),
        (0x4EEB_C549, Opcode::SimdFpFminnmVec, "fminnm"),
        (0x6E22_F420, Opcode::SimdFpFmaxp, "fmaxp"),
        (0x6EE5_F483, Opcode::SimdFpFminp, "fminp"),
        (0x6E28_C4E6, Opcode::SimdFpFmaxnmp, "fmaxnmp"),
        (0x6EEB_C549, Opcode::SimdFpFminnmp, "fminnmp"),
    ];
    assert_decode_cases(&cases);

    let fmax = decode(0x4E22_F420).unwrap();
    assert_eq!((fmax.rd, fmax.rn, fmax.rm), (0, 1, 2));
    assert_eq!((fmax.imm, fmax.size), (4, 16));
    let fmin = decode(0x4EE5_F483).unwrap();
    assert_eq!((fmin.rd, fmin.rn, fmin.rm), (3, 4, 5));
    assert_eq!((fmin.imm, fmin.size), (8, 16));
    let fmaxp = decode(0x6E22_F420).unwrap();
    assert_eq!((fmaxp.rd, fmaxp.rn, fmaxp.rm), (0, 1, 2));
    assert_eq!((fmaxp.imm, fmaxp.size), (4, 16));

    for raw in [0x0E62_F420, 0x2E62_F420, 0x0E62_C420, 0x2E62_C420] {
        assert!(decode(raw).is_none(), "raw=0x{raw:08x}");
    }
}

#[test]
fn decode_simd_fp_pairwise_scalar_minmax_forms_cross_checked_with_disarm64() {
    let cases = [
        (0x7E30_F820, Opcode::SimdFpFmaxp, "fmaxp"),
        (0x7EB0_F862, Opcode::SimdFpFminp, "fminp"),
        (0x7E30_C8A4, Opcode::SimdFpFmaxnmp, "fmaxnmp"),
        (0x7EB0_C8E6, Opcode::SimdFpFminnmp, "fminnmp"),
        (0x7E70_F928, Opcode::SimdFpFmaxp, "fmaxp"),
        (0x7EF0_F96A, Opcode::SimdFpFminp, "fminp"),
        (0x7E70_C9AC, Opcode::SimdFpFmaxnmp, "fmaxnmp"),
        (0x7EF0_C9EE, Opcode::SimdFpFminnmp, "fminnmp"),
    ];
    assert_decode_cases(&cases);

    let fmaxp = decode(0x7E30_F820).unwrap();
    assert_eq!((fmaxp.rd, fmaxp.rn, fmaxp.rm), (0, 1, 0));
    assert_eq!((fmaxp.imm, fmaxp.size), (4, 4));
    let fminnmp = decode(0x7EF0_C9EE).unwrap();
    assert_eq!((fminnmp.rd, fminnmp.rn, fminnmp.rm), (14, 15, 0));
    assert_eq!((fminnmp.imm, fminnmp.size), (8, 8));
}
