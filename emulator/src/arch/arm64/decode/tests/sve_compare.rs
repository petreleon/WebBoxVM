use super::*;

#[test]
fn decode_sve_unsigned_higher_same_compare_forms() {
    let cases = [
        (0x2401_0000, Opcode::SveCmpHs, "cmphs"),
        (0x2441_0000, Opcode::SveCmpHs, "cmphs"),
        (0x2498_0342, Opcode::SveCmpHs, "cmphs"),
        (0x24DC_0FC2, Opcode::SveCmpHs, "cmphs"),
        (0x2420_C483, Opcode::SveCmpHsImm, "cmphs"),
        (0x24EF_C302, Opcode::SveCmpHsImm, "cmphs"),
        (0x2401_0010, Opcode::SveCmpHi, "cmphi"),
        (0x2420_8430, Opcode::SveCmpHiImm, "cmphi"),
        (0x249E_A003, Opcode::SveCmpEq, "cmpeq"),
        (0x25C0_8FA3, Opcode::SveCmpEqImm, "cmpeq"),
        (0x241E_A013, Opcode::SveCmpNe, "cmpne"),
        (0x251F_8030, Opcode::SveCmpNeImm, "cmpne"),
    ];

    for (raw, expected, mnemonic) in cases {
        assert_disarm64_mnemonic(raw, mnemonic);
        assert_eq!(decode(raw).unwrap().op, expected, "raw=0x{raw:08x}");
    }

    let vec = decode(0x2498_0342).unwrap(); // cmphs p2.s, p0/z, z26.s, z24.s
    assert_eq!(vec.rd, 2);
    assert_eq!(vec.rn, 26);
    assert_eq!(vec.rm, 24);
    assert_eq!(vec.cond, 0);
    assert_eq!(vec.size, 4);

    let imm = decode(0x24EF_C302).unwrap(); // cmphs p2.d, p0/z, z24.d, #0x3f
    assert_eq!(imm.rd, 2);
    assert_eq!(imm.rn, 24);
    assert_eq!(imm.imm, 63);
    assert_eq!(imm.cond, 0);
    assert_eq!(imm.size, 8);

    let eq = decode(0x25C0_8FA3).unwrap(); // cmpeq p3.d, p3/z, z29.d, #0
    assert_eq!(
        (eq.op, eq.rd, eq.rn, eq.imm, eq.cond),
        (Opcode::SveCmpEqImm, 3, 29, 0, 3)
    );

    let ne = decode(0x251F_8030).unwrap(); // cmpne p0.b, p0/z, z1.b, #-1
    assert_eq!(
        (ne.op, ne.rd, ne.rn, ne.imm as i64),
        (Opcode::SveCmpNeImm, 0, 1, -1)
    );
}
