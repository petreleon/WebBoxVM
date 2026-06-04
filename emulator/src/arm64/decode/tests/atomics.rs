use super::*;

#[test]
fn decode_lse_caspal() {
    let instr = decode(0x4860_FC82).unwrap(); // caspal x0, x1, x2, x3, [x4]
    assert_eq!(instr.op, Opcode::Casp);
    assert_eq!(instr.rd, 0);
    assert_eq!(instr.rm, 2);
    assert_eq!(instr.rn, 4);
    assert_eq!(instr.size, 8);
}

#[test]
fn decode_lse_ldaddal() {
    let instr = decode(0xB8E1_0001).unwrap(); // ldaddal w1, w1, [x0]
    assert_eq!(instr.op, Opcode::Atomic);
    assert_eq!(instr.rd, 1);
    assert_eq!(instr.rm, 1);
    assert_eq!(instr.rn, 0);
    assert_eq!(instr.imm, 0);
    assert_eq!(instr.size, 4);
}

#[test]
fn decode_lse128_pair_atomics() {
    for (raw, op) in [
        (0x1921_80C0, 0x8), // swpp x0, x1, [x6]
        (0x19A1_80C0, 0x8), // swppa x0, x1, [x6]
        (0x19E1_80C0, 0x8), // swppal x0, x1, [x6]
        (0x1921_30C0, 0x3), // ldsetp x0, x1, [x6]
        (0x19A1_30C0, 0x3), // ldsetpa x0, x1, [x6]
        (0x19E1_30C0, 0x3), // ldsetpal x0, x1, [x6]
        (0x1921_10C0, 0x1), // ldclrp x0, x1, [x6]
        (0x19A1_10C0, 0x1), // ldclrpa x0, x1, [x6]
        (0x19E1_10C0, 0x1), // ldclrpal x0, x1, [x6]
    ] {
        let instr = decode(raw).unwrap();
        assert_eq!(instr.op, Opcode::AtomicPair);
        assert_eq!(instr.rd, 0);
        assert_eq!(instr.rm, 1);
        assert_eq!(instr.rn, 6);
        assert_eq!(instr.imm, op);
        assert_eq!(instr.size, 8);
        assert!(instr.sf);
    }

    let ldsetp = decode(0x1923_3002).unwrap(); // ldsetp x2, x3, [x0]
    assert_eq!(ldsetp.op, Opcode::AtomicPair);
    assert_eq!(ldsetp.rd, 2);
    assert_eq!(ldsetp.rm, 3);
    assert_eq!(ldsetp.rn, 0);

    assert!(decode(0x1921_80DF).is_none()); // Rt == XZR
    assert!(decode(0x193F_80C0).is_none()); // Rt2 == XZR
    assert!(decode(0x1920_80C0).is_none()); // Rt == Rt2 is constrained unpredictable
}
