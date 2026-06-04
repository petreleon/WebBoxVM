use super::*;

#[test]
fn decode_simd_reduce_across_forms_cross_checked_with_disarm64() {
    let cases = [
        (0x0E30_A820, Opcode::SimdSmaxv, "smaxv"),
        (0x4E30_A820, Opcode::SimdSmaxv, "smaxv"),
        (0x0E71_A820, Opcode::SimdSminv, "sminv"),
        (0x4EB1_A820, Opcode::SimdSminv, "sminv"),
        (0x2E31_A820, Opcode::SimdUminv, "uminv"),
        (0x6E71_A820, Opcode::SimdUminv, "uminv"),
        (0x6E30_F820, Opcode::SimdFpFmaxv, "fmaxv"),
        (0x6EB0_F862, Opcode::SimdFpFminv, "fminv"),
        (0x6E30_C8A4, Opcode::SimdFpFmaxnmv, "fmaxnmv"),
        (0x6EB0_C8E6, Opcode::SimdFpFminnmv, "fminnmv"),
    ];
    assert_decode_cases(&cases);

    let smin_words = decode(0x4EB1_A820).unwrap();
    assert_eq!((smin_words.rd, smin_words.rn, smin_words.rm), (0, 1, 0));
    assert_eq!((smin_words.imm, smin_words.size), (4, 16));
    let umin_half = decode(0x6E71_A820).unwrap();
    assert_eq!((umin_half.imm, umin_half.size), (2, 16));
    let fmaxv = decode(0x6E30_F820).unwrap();
    assert_eq!((fmaxv.rd, fmaxv.rn, fmaxv.rm), (0, 1, 0));
    assert_eq!((fmaxv.imm, fmaxv.size), (4, 16));
    let fminnmv = decode(0x6EB0_C8E6).unwrap();
    assert_eq!((fminnmv.rd, fminnmv.rn), (6, 7));
    for invalid in [0x0EB0_A820, 0x0EF0_A820, 0x4EF1_A820, 0x6EF1_A820] {
        assert!(decode(invalid).is_none(), "raw=0x{invalid:08x}");
    }
}
