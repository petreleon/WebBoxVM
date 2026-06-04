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
}
