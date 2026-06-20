use super::*;

#[test]
fn decode_sve_zip_vectors_cross_checked_with_disarm64() {
    assert_decode_cases(&[
        (0x0520_6000, Opcode::SveZip1, "zip1"),
        (0x0560_641F, Opcode::SveZip2, "zip2"),
        (0x05A4_6011, Opcode::SveZip1, "zip1"),
        (0x05A4_6412, Opcode::SveZip2, "zip2"),
        (0x05F3_6220, Opcode::SveZip1, "zip1"),
        (0x05F3_6624, Opcode::SveZip2, "zip2"),
        (0x05BB_6BDE, Opcode::SveUzp1, "uzp1"),
        (0x05E0_6C00, Opcode::SveUzp2, "uzp2"),
    ]);

    let word = decode(0x05A4_6011).unwrap(); // zip1 z17.s, z0.s, z4.s
    assert_eq!(
        (word.rd, word.rn, word.rm, word.cond, word.size),
        (17, 0, 4, 0xFF, 4)
    );

    let dword = decode(0x05F3_6624).unwrap(); // zip2 z4.d, z17.d, z19.d
    assert_eq!(
        (dword.rd, dword.rn, dword.rm, dword.cond, dword.size),
        (4, 17, 19, 0xFF, 8)
    );

    let uzp = decode(0x05BB_6BDE).unwrap(); // uzp1 z30.s, z30.s, z27.s
    assert_eq!(
        (uzp.rd, uzp.rn, uzp.rm, uzp.cond, uzp.size),
        (30, 30, 27, 0xFF, 4)
    );
}

#[test]
fn decode_sve_tbl_vectors_cross_checked_with_disarm64() {
    assert_decode_cases(&[
        (0x0520_3000, Opcode::SveTbl, "tbl"),
        (0x0564_2811, Opcode::SveTbl, "tbl"),
        (0x05A7_2822, Opcode::SveTbl, "tbl"),
        (0x05F3_2824, Opcode::SveTbl, "tbl"),
        (0x053F_3063, Opcode::SveTbl, "tbl"),
    ]);

    let single = decode(0x053F_3063).unwrap();
    assert_eq!(
        (single.rd, single.rn, single.rm, single.imm, single.size),
        (3, 3, 31, 1, 1)
    );

    let pair = decode(0x05F3_2824).unwrap();
    assert_eq!(
        (pair.rd, pair.rn, pair.rm, pair.imm, pair.size),
        (4, 1, 19, 2, 8)
    );
}
