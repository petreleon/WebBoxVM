use super::*;

#[test]
fn decode_simd_fp_pairwise_add_forms_cross_checked_with_disarm64() {
    let cases = [
        (0x2E22_D420, Opcode::SimdFpAddp, "faddp"),
        (0x6E25_D483, Opcode::SimdFpAddp, "faddp"),
        (0x6E68_D4E6, Opcode::SimdFpAddp, "faddp"),
        (0x7E30_D949, Opcode::SimdFpAddp, "faddp"),
        (0x7E70_D98B, Opcode::SimdFpAddp, "faddp"),
    ];
    assert_decode_cases(&cases);

    let v2s = decode(0x2E22_D420).unwrap();
    assert_eq!((v2s.rd, v2s.rn, v2s.rm), (0, 1, 2));
    assert_eq!((v2s.imm, v2s.size), (4, 8));

    let v2d = decode(0x6E68_D4E6).unwrap();
    assert_eq!((v2d.rd, v2d.rn, v2d.rm), (6, 7, 8));
    assert_eq!((v2d.imm, v2d.size), (8, 16));

    let scalar_s = decode(0x7E30_D949).unwrap();
    assert_eq!((scalar_s.rd, scalar_s.rn, scalar_s.rm), (9, 10, 0));
    assert_eq!((scalar_s.imm, scalar_s.size), (4, 4));

    let scalar_d = decode(0x7E70_D98B).unwrap();
    assert_eq!((scalar_d.rd, scalar_d.rn, scalar_d.rm), (11, 12, 0));
    assert_eq!((scalar_d.imm, scalar_d.size), (8, 8));

    assert!(decode(0x2E60_D400).is_none());
}
