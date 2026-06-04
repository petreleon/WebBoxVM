use super::*;

#[test]
fn decode_simd_signed_register_compares_cross_checked_with_disarm64() {
    let cases = [
        (0x0E22_3C20, Opcode::SimdCmgeReg, "cmge"),
        (0x0E7A_3C84, Opcode::SimdCmgeReg, "cmge"),
        (0x4EBD_3F39, Opcode::SimdCmgeReg, "cmge"),
        (0x4EFE_3FE3, Opcode::SimdCmgeReg, "cmge"),
        (0x0E22_3420, Opcode::SimdCmgtReg, "cmgt"),
        (0x0E7A_3484, Opcode::SimdCmgtReg, "cmgt"),
        (0x4EBD_3739, Opcode::SimdCmgtReg, "cmgt"),
        (0x4EFE_37E3, Opcode::SimdCmgtReg, "cmgt"),
    ];
    assert_decode_cases(&cases);

    let halfwords = decode(0x0E7A_3C84).unwrap();
    assert_eq!(halfwords.rd, 4);
    assert_eq!(halfwords.rn, 4);
    assert_eq!(halfwords.rm, 26);
    assert_eq!(halfwords.imm, 2);
    assert_eq!(halfwords.size, 8);

    let doublewords = decode(0x4EFE_37E3).unwrap();
    assert_eq!(doublewords.imm, 8);
    assert_eq!(doublewords.size, 16);
    assert!(decode(0x0EE2_3420).is_none());
    assert!(decode(0x0EE2_3C20).is_none());
}
