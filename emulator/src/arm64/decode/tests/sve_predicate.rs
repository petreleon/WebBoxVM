use super::*;

#[test]
fn decode_sve_predicate_forms() {
    let cases = [
        (0x2518_E3E3, Opcode::SvePtrue, "ptrue"),
        (0x25D8_E3E1, Opcode::SvePtrue, "ptrue"),
        (0x2550_C060, Opcode::SvePtest, "ptest"),
        (0x2583_4443, Opcode::SvePredOrr, "orr"),
        (0x250F_480F, Opcode::SvePredAnd, "and"),
        (0x2543_4447, Opcode::SvePredAnd, "ands"),
        (0x25C3_4448, Opcode::SvePredOrr, "orrs"),
    ];
    for (raw, expected, mnemonic) in cases {
        assert_disarm64_mnemonic(raw, mnemonic);
        assert_eq!(decode(raw).unwrap().op, expected, "raw=0x{raw:08x}");
    }

    let ptrue = decode(0x2558_E064).unwrap(); // ptrue p4.h, vl3
    assert_eq!(ptrue.op, Opcode::SvePtrue);
    assert_eq!(ptrue.rd, 4);
    assert_eq!(ptrue.cond, 3);
    assert_eq!(ptrue.size, 2);

    let ptest = decode(0x2550_C460).unwrap(); // ptest p1, p3.b
    assert_eq!(ptest.op, Opcode::SvePtest);
    assert_eq!(ptest.rd, 1);
    assert_eq!(ptest.rn, 3);

    let orr = decode(0x2583_4443).unwrap(); // orr p3.b, p1/z, p2.b, p3.b
    assert_eq!(orr.rd, 3);
    assert_eq!(orr.rn, 2);
    assert_eq!(orr.rm, 3);
    assert_eq!(orr.cond, 1);
    assert!(!orr.sf);

    let ands = decode(0x2543_4447).unwrap(); // ands p7.b, p1/z, p2.b, p3.b
    assert_eq!(ands.rd, 7);
    assert_eq!(ands.rn, 2);
    assert_eq!(ands.rm, 3);
    assert_eq!(ands.cond, 1);
    assert!(ands.sf);
}
