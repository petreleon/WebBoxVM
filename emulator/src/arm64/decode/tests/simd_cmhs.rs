use super::*;

#[test]
fn decode_simd_cmhs_vector_lane_widths() {
    assert_decode_cases(&[
        (0x2E22_3C20, Opcode::SimdCmhsReg, "cmhs"),
        (0x2E7A_3C84, Opcode::SimdCmhsReg, "cmhs"),
        (0x6EBD_3F39, Opcode::SimdCmhsReg, "cmhs"),
        (0x6EFE_3FE3, Opcode::SimdCmhsReg, "cmhs"),
    ]);

    let bytes = decode(0x2E22_3C20).unwrap();
    assert_eq!(bytes.imm, 1);
    assert_eq!(bytes.size, 8);
    let halfwords = decode(0x2E7A_3C84).unwrap();
    assert_eq!(halfwords.rd, 4);
    assert_eq!(halfwords.rn, 4);
    assert_eq!(halfwords.rm, 26);
    assert_eq!(halfwords.imm, 2);
    assert_eq!(halfwords.size, 8);
    let words = decode(0x6EBD_3F39).unwrap();
    assert_eq!(words.rd, 25);
    assert_eq!(words.rn, 25);
    assert_eq!(words.rm, 29);
    assert_eq!(words.imm, 4);
    assert_eq!(words.size, 16);
    let doublewords = decode(0x6EFE_3FE3).unwrap();
    assert_eq!(doublewords.rd, 3);
    assert_eq!(doublewords.rn, 31);
    assert_eq!(doublewords.rm, 30);
    assert_eq!(doublewords.imm, 8);
    assert_eq!(doublewords.size, 16);
    assert!(decode(0x2EE2_3C20).is_none());
}
