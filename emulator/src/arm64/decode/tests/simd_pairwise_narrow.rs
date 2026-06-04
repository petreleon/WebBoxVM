use super::*;

#[test]
fn decode_simd_pairwise_narrow_cases_cross_checked_with_disarm64() {
    let cases = [
        (0x6E22_3C20, Opcode::SimdCmhsReg, "cmhs"),
        (0x0E28_40E6, Opcode::SimdAddhn, "addhn"),
        (0x6E25_A483, Opcode::SimdUmaxp, "umaxp"),
    ];

    assert_decode_cases(&cases);

    let addhn = decode(0x0E28_40E6).unwrap();
    assert_eq!(addhn.rd, 6);
    assert_eq!(addhn.rn, 7);
    assert_eq!(addhn.rm, 8);
    assert_eq!(addhn.size, 8);

    assert_disarm64_mnemonic(0x4E2B_4149, "addhn2");
    assert!(decode(0x4E2B_4149).is_none());
}
