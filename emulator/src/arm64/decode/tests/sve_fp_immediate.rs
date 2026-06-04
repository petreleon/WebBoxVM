use super::*;

#[test]
fn decode_sve_fadd_immediate_forms() {
    let cases = [
        (0x6558_8000, Opcode::SveFpAddImm, "fadd"),
        (0x6558_8421, Opcode::SveFpAddImm, "fadd"),
        (0x6598_8802, Opcode::SveFpAddImm, "fadd"),
        (0x6598_8C23, Opcode::SveFpAddImm, "fadd"),
        (0x65D8_9004, Opcode::SveFpAddImm, "fadd"),
        (0x65D8_9425, Opcode::SveFpAddImm, "fadd"),
        (0x65D8_803E, Opcode::SveFpAddImm, "fadd"),
    ];

    for (raw, expected, mnemonic) in cases {
        assert_disarm64_mnemonic(raw, mnemonic);
        assert_eq!(decode(raw).unwrap().op, expected, "raw=0x{raw:08x}");
    }

    let fadd = decode(0x6598_8C23).unwrap(); // fadd z3.s, p3/m, z3.s, #1
    assert_eq!(fadd.rd, 3);
    assert_eq!(fadd.rn, 3);
    assert_eq!(fadd.imm, 1);
    assert_eq!(fadd.cond, 3);
    assert_eq!(fadd.size, 4);

    let half = decode(0x6558_8000).unwrap(); // fadd z0.h, p0/m, z0.h, #0.5
    assert_eq!(half.imm, 0);
    assert_eq!(half.size, 2);

    assert!(decode(0x6518_8000).is_none());
}
