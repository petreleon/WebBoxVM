use super::*;

#[test]
fn decode_sve_revh_forms_cross_checked_with_disarm64() {
    assert_decode_cases(&[
        (0x05A5_8063, Opcode::SveRevh, "revh"),
        (0x05A5_80E7, Opcode::SveRevh, "revh"),
        (0x05A5_816B, Opcode::SveRevh, "revh"),
        (0x05A5_81EF, Opcode::SveRevh, "revh"),
        (0x05E5_8063, Opcode::SveRevh, "revh"),
    ]);

    let word = decode(0x05A5_8063).unwrap();
    assert_eq!((word.rd, word.rn, word.cond), (3, 3, 0));
    assert_eq!((word.imm, word.size), (2, 4));

    let dword = decode(0x05E5_8063).unwrap();
    assert_eq!((dword.rd, dword.rn, dword.cond), (3, 3, 0));
    assert_eq!((dword.imm, dword.size), (2, 8));

    assert!(decode(0x0525_8000).is_none());
    assert!(decode(0x0565_8000).is_none());
}
