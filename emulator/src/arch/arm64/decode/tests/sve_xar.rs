use super::*;

#[test]
fn decode_sve_xar_vectors_cross_checked_with_disarm64() {
    assert_decode_cases(&[
        (0x042D_34C5, Opcode::SveXar, "xar"),
        (0x0433_3421, Opcode::SveXar, "xar"),
        (0x0470_3403, Opcode::SveXar, "xar"),
        (0x04E0_3462, Opcode::SveXar, "xar"),
    ]);

    let word = decode(0x0470_3403).unwrap(); // xar z3.s, z3.s, z0.s, #16
    assert_eq!(
        (word.rd, word.rn, word.rm, word.imm, word.size),
        (3, 3, 0, 16, 4)
    );

    let dword = decode(0x04E0_3462).unwrap(); // xar z2.d, z2.d, z3.d, #32
    assert_eq!(
        (dword.rd, dword.rn, dword.rm, dword.imm, dword.size),
        (2, 2, 3, 32, 8)
    );

    assert!(decode(0x0420_3400).is_none());
}
