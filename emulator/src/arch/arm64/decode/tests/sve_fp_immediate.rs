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
        (0x6599_8000, Opcode::SveFpSub, "fsub"),
        (0x6599_8020, Opcode::SveFpSub, "fsub"),
        (0x65D9_803A, Opcode::SveFpSub, "fsub"),
        (0x659B_8000, Opcode::SveFpSubr, "fsubr"),
        (0x659B_8020, Opcode::SveFpSubr, "fsubr"),
        (0x65DB_801D, Opcode::SveFpSubr, "fsubr"),
        (0x0592_C01D, Opcode::SveFpCpyImm, "fcpy"),
        (0x05D3_CE1F, Opcode::SveFpCpyImm, "fcpy"),
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

    let fsubr = decode(0x659B_8020).unwrap(); // fsubr z0.s, p0/m, z0.s, #1
    assert_eq!(fsubr.op, Opcode::SveFpSubr);
    assert_eq!(fsubr.rm, 0xFF);
    assert_eq!(fsubr.imm, 1);

    let fcpy = decode(0x05D3_CE1F).unwrap(); // fcpy z31.d, p3/m, #1
    assert_eq!(fcpy.rd, 31);
    assert_eq!(fcpy.cond, 3);
    assert_eq!(fcpy.imm, 0x70);
    assert_eq!(fcpy.size, 8);

    assert!(decode(0x6518_8000).is_none());
    assert!(decode(0x0510_C020).is_none());
}
