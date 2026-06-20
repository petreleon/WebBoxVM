use super::*;

#[test]
fn decode_sve_ftmad_forms_cross_checked_with_disarm64() {
    assert_decode_cases(&[
        (0x6550_83C1, Opcode::SveFpFtmad, "ftmad"),
        (0x6594_83C1, Opcode::SveFpFtmad, "ftmad"),
        (0x65D7_83C1, Opcode::SveFpFtmad, "ftmad"),
    ]);

    let half = decode(0x6550_83C1).unwrap(); // ftmad z1.h, z1.h, z30.h, #0
    assert_eq!(
        (half.rd, half.rn, half.rm, half.imm, half.cond, half.size),
        (1, 1, 30, 0, 0xFF, 2)
    );

    let single = decode(0x6594_83C1).unwrap(); // ftmad z1.s, z1.s, z30.s, #4
    assert_eq!(
        (
            single.rd,
            single.rn,
            single.rm,
            single.imm,
            single.cond,
            single.size
        ),
        (1, 1, 30, 4, 0xFF, 4)
    );

    let double = decode(0x65D7_83C1).unwrap(); // ftmad z1.d, z1.d, z30.d, #7
    assert_eq!(
        (
            double.rd,
            double.rn,
            double.rm,
            double.imm,
            double.cond,
            double.size
        ),
        (1, 1, 30, 7, 0xFF, 8)
    );

    assert!(decode(0x6510_8000).is_none());
}

#[test]
fn decode_sve_ftsmul_and_ftssel_forms_cross_checked_with_disarm64() {
    assert_decode_cases(&[
        (0x6543_0C41, Opcode::SveFpFtsmul, "ftsmul"),
        (0x6583_0C41, Opcode::SveFpFtsmul, "ftsmul"),
        (0x65C3_0C41, Opcode::SveFpFtsmul, "ftsmul"),
        (0x0463_B041, Opcode::SveFpFtssel, "ftssel"),
        (0x04A3_B041, Opcode::SveFpFtssel, "ftssel"),
        (0x04E3_B041, Opcode::SveFpFtssel, "ftssel"),
    ]);

    let smul = decode(0x6583_0C41).unwrap(); // ftsmul z1.s, z2.s, z3.s
    assert_eq!(
        (smul.rd, smul.rn, smul.rm, smul.cond, smul.size),
        (1, 2, 3, 0xFF, 4)
    );

    let ssel = decode(0x04E3_B041).unwrap(); // ftssel z1.d, z2.d, z3.d
    assert_eq!(
        (ssel.rd, ssel.rn, ssel.rm, ssel.cond, ssel.size),
        (1, 2, 3, 0xFF, 8)
    );

    assert!(decode(0x6503_0C41).is_none());
    assert!(decode(0x0423_B041).is_none());
}
