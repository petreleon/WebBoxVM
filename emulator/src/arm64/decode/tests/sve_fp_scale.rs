use super::*;

#[test]
fn decode_sve_fscale_forms_cross_checked_with_disarm64() {
    assert_decode_cases(&[
        (0x6549_8FE1, Opcode::SveFpFscale, "fscale"),
        (0x6589_8FE1, Opcode::SveFpFscale, "fscale"),
        (0x65C9_8FE1, Opcode::SveFpFscale, "fscale"),
    ]);

    let half = decode(0x6549_8FE1).unwrap(); // fscale z1.h, p3/m, z1.h, z31.h
    assert_eq!(
        (half.rd, half.rn, half.rm, half.cond, half.size),
        (1, 1, 31, 3, 2)
    );

    let single = decode(0x6589_8FE1).unwrap(); // fscale z1.s, p3/m, z1.s, z31.s
    assert_eq!(
        (single.rd, single.rn, single.rm, single.cond, single.size),
        (1, 1, 31, 3, 4)
    );

    let double = decode(0x65C9_8FE1).unwrap(); // fscale z1.d, p3/m, z1.d, z31.d
    assert_eq!(
        (double.rd, double.rn, double.rm, double.cond, double.size),
        (1, 1, 31, 3, 8)
    );

    assert!(decode(0x6509_8FE1).is_none());
}
