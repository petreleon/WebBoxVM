use super::*;

#[test]
fn decode_sve_fexpa_forms_cross_checked_with_disarm64() {
    assert_decode_cases(&[
        (0x0460_B800, Opcode::SveFpFexpa, "fexpa"),
        (0x0460_B821, Opcode::SveFpFexpa, "fexpa"),
        (0x04A0_B800, Opcode::SveFpFexpa, "fexpa"),
        (0x04E0_B800, Opcode::SveFpFexpa, "fexpa"),
        (0x04E0_BBFF, Opcode::SveFpFexpa, "fexpa"),
    ]);

    let half = decode(0x0460_B821).unwrap(); // fexpa z1.h, z1.h
    assert_eq!((half.rd, half.rn, half.cond, half.size), (1, 1, 0xFF, 2));

    let single = decode(0x04A0_B83E).unwrap(); // fexpa z30.s, z1.s
    assert_eq!(
        (single.rd, single.rn, single.cond, single.size),
        (30, 1, 0xFF, 4)
    );

    let double = decode(0x04E0_BBFF).unwrap(); // fexpa z31.d, z31.d
    assert_eq!(
        (double.rd, double.rn, double.cond, double.size),
        (31, 31, 0xFF, 8)
    );

    assert!(decode(0x0420_B800).is_none());
}
