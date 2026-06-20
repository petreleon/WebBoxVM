use super::*;

#[test]
fn decode_sve_dup_immediate_and_indexed_forms_cross_checked_with_disarm64() {
    let cases = [
        (0x2538_C020, Opcode::SveDupImm, "dup"),
        (0x2538_DFE1, Opcode::SveDupImm, "dup"),
        (0x2578_E024, Opcode::SveDupImm, "dup"),
        (0x25B8_FFC5, Opcode::SveDupImm, "dup"),
        (0x0527_2128, Opcode::SveDupElem, "dup"),
        (0x056C_21AC, Opcode::SveDupElem, "dup"),
        (0x05F8_22F6, Opcode::SveDupElem, "dup"),
        (0x05D2_401F, Opcode::SveCpyImm, "cpy"),
        (0x05D2_001F, Opcode::SveCpyImm, "cpy"),
        (0x05E8_A41A, Opcode::SveCpyGpr, "cpy"),
    ];
    assert_decode_cases(&cases);

    let shifted = decode(0x25B8_FFC5).unwrap(); // dup z5.s, #-2, lsl #8
    assert_eq!((shifted.rd, shifted.size), (5, 4));
    assert_eq!(shifted.imm as i64, -512);

    let indexed = decode(0x056C_21AC).unwrap(); // dup z12.s, z13.s[5]
    assert_eq!(
        (indexed.rd, indexed.rn, indexed.size, indexed.imm),
        (12, 13, 4, 5)
    );

    let cpy_imm = decode(0x05D2_401F).unwrap(); // cpy z31.d, p2/m, #0
    assert_eq!((cpy_imm.rd, cpy_imm.cond, cpy_imm.size), (31, 2, 8));
    assert!(cpy_imm.sf);

    let cpy_gpr = decode(0x05E8_A41A).unwrap(); // cpy z26.d, p1/m, x0
    assert_eq!((cpy_gpr.rd, cpy_gpr.rn, cpy_gpr.cond), (26, 0, 1));

    assert!(decode(0x2538_E000).is_none());
    assert!(decode(0x0510_2000).is_none());
    assert!(decode(0x0520_2000).is_none());
}
