use super::*;

#[test]
fn decode_sve_halfword_store_forms_cross_checked_with_disarm64() {
    let cases = [
        (0xE4A0_ECA7, 7, 5, 0xFF, 0, 3, 2),
        (0xE4C1_ECA7, 7, 5, 0xFF, 1, 3, 4),
        (0xE4AE_E082, 2, 4, 0xFF, -2, 0, 2),
        (0xE4FE_5C1F, 31, 0, 30, 0, 7, 8),
    ];

    for &(raw, rd, rn, rm, imm, pred, size) in &cases {
        assert_disarm64_mnemonic(raw, "st1h");
        let instr = decode(raw).unwrap();
        assert_eq!(instr.op, Opcode::SveSt1h);
        assert_eq!((instr.rd, instr.rn, instr.rm), (rd, rn, rm));
        assert_eq!(instr.imm as i64, imm);
        assert_eq!((instr.cond, instr.size), (pred, size));
    }
}

#[test]
fn decode_sve_halfword_store_leaves_vector_offset_form_unmapped() {
    assert_disarm64_mnemonic(0xE4BF_CAB2, "st1h");
    assert!(decode(0xE4BF_CAB2).is_none());
}
