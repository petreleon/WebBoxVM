use super::*;

#[test]
fn decode_sve_fp_abs_and_neg_forms() {
    let cases = [
        (0x045C_A020, Opcode::SveFpAbs, "fabs"),
        (0x049C_AC82, Opcode::SveFpAbs, "fabs"),
        (0x04DC_B8E5, Opcode::SveFpAbs, "fabs"),
        (0x049D_A528, Opcode::SveFpNeg, "fneg"),
        (0x04DD_A96A, Opcode::SveFpNeg, "fneg"),
        (0x04DC_A01C, Opcode::SveFpAbs, "fabs"),
        (0x049D_A35D, Opcode::SveFpNeg, "fneg"),
        (0x6580_A3DE, Opcode::SveFpFrintn, "frintn"),
        (0x65C4_A39C, Opcode::SveFpFrinta, "frinta"),
        (0x6583_BFFE, Opcode::SveFpFrintz, "frintz"),
        (0x65CD_ABFB, Opcode::SveFpSqrt, "fsqrt"),
    ];

    for (raw, expected, mnemonic) in cases {
        assert_disarm64_mnemonic(raw, mnemonic);
        assert_eq!(decode(raw).unwrap().op, expected, "raw=0x{raw:08x}");
    }

    let abs = decode(0x049C_AC82).unwrap(); // fabs z2.s, p3/m, z4.s
    assert_eq!(abs.rd, 2);
    assert_eq!(abs.rn, 4);
    assert_eq!(abs.cond, 3);
    assert_eq!(abs.size, 4);

    let neg = decode(0x04DD_A96A).unwrap(); // fneg z10.d, p2/m, z11.d
    assert_eq!(neg.rd, 10);
    assert_eq!(neg.rn, 11);
    assert_eq!(neg.cond, 2);
    assert_eq!(neg.size, 8);

    let frintz = decode(0x6583_BFFE).unwrap(); // frintz z30.s, p7/m, z31.s
    assert_eq!(frintz.rd, 30);
    assert_eq!(frintz.rn, 31);
    assert_eq!(frintz.cond, 7);
    assert_eq!(frintz.size, 4);

    let sqrt = decode(0x65CD_ABFB).unwrap(); // fsqrt z27.d, p2/m, z31.d
    assert_eq!(sqrt.rd, 27);
    assert_eq!(sqrt.rn, 31);
    assert_eq!(sqrt.cond, 2);
    assert_eq!(sqrt.size, 8);

    assert!(decode(0x041C_A000).is_none());
    assert!(decode(0x6540_A000).is_none());
}
