use super::*;

#[test]
fn decode_simd_pairwise_narrow_cases_cross_checked_with_disarm64() {
    let cases = [
        (0x6E22_3C20, Opcode::SimdCmhsReg, "cmhs"),
        (0x0E28_40E6, Opcode::SimdAddhn, "addhn"),
        (0x4E2B_4149, Opcode::SimdAddhn2, "addhn2"),
        (0x2E28_40E6, Opcode::SimdRaddhn, "raddhn"),
        (0x6E2B_4149, Opcode::SimdRaddhn2, "raddhn2"),
        (0x0E7E_6002, Opcode::SimdSubhn, "subhn"),
        (0x4E60_6000, Opcode::SimdSubhn2, "subhn2"),
        (0x2E7E_6002, Opcode::SimdRsubhn, "rsubhn"),
        (0x6E60_6000, Opcode::SimdRsubhn2, "rsubhn2"),
        (0x0EA5_6042, Opcode::SimdSubhn, "subhn"),
        (0x4F0A_87E2, Opcode::SimdShrn2, "shrn2"),
        (0x0F0A_8FFF, Opcode::SimdRshrn, "rshrn"),
        (0x4F0A_8FE2, Opcode::SimdRshrn2, "rshrn2"),
        (0x6E25_A483, Opcode::SimdUmaxp, "umaxp"),
    ];

    assert_decode_cases(&cases);

    let addhn = decode(0x0E28_40E6).unwrap();
    assert_eq!(addhn.rd, 6);
    assert_eq!(addhn.rn, 7);
    assert_eq!(addhn.rm, 8);
    assert_eq!(addhn.imm, 1);
    assert_eq!(addhn.size, 8);

    let subhn = decode(0x0E7E_6002).unwrap();
    assert_eq!(subhn.rd, 2);
    assert_eq!(subhn.rn, 0);
    assert_eq!(subhn.rm, 30);
    assert_eq!(subhn.imm, 2);
    assert_eq!(subhn.size, 8);

    let addhn2 = decode(0x4E2B_4149).unwrap();
    assert_eq!(addhn2.rd, 9);
    assert_eq!(addhn2.rn, 10);
    assert_eq!(addhn2.rm, 11);
    assert_eq!(addhn2.imm, 1);
    assert_eq!(addhn2.size, 16);

    let shrn2 = decode(0x4F0A_87E2).unwrap();
    assert_eq!(shrn2.rd, 2);
    assert_eq!(shrn2.rn, 31);
    assert_eq!(shrn2.imm, 6);
    assert_eq!(shrn2.cond, 1);
    assert_eq!(shrn2.size, 16);
    assert!(decode(0x0EE0_6000).is_none());
    assert!(decode(0x4F60_8422).is_none());
}
