use super::*;

#[test]
fn decode_sve_signed_int_fp_convert_forms_cross_checked_with_disarm64() {
    let cases = [
        (0x6594_AFFF, Opcode::SveScvtf, "scvtf"),
        (0x65D0_A020, Opcode::SveScvtf, "scvtf"),
        (0x65D4_A020, Opcode::SveScvtf, "scvtf"),
        (0x65D6_A3DE, Opcode::SveScvtf, "scvtf"),
        (0x659C_A3FF, Opcode::SveFcvtzs, "fcvtzs"),
        (0x65DC_A020, Opcode::SveFcvtzs, "fcvtzs"),
        (0x65D8_A020, Opcode::SveFcvtzs, "fcvtzs"),
        (0x65DE_A39A, Opcode::SveFcvtzs, "fcvtzs"),
    ];

    for (raw, expected, mnemonic) in cases {
        assert_disarm64_mnemonic(raw, mnemonic);
        assert_eq!(decode(raw).unwrap().op, expected, "raw=0x{raw:08x}");
    }

    let scvtf = decode(0x65D6_A3DE).unwrap(); // scvtf z30.d, p0/m, z30.d
    assert_eq!((scvtf.rd, scvtf.rn, scvtf.cond), (30, 30, 0));
    assert_eq!((scvtf.size, scvtf.imm, scvtf.rm), (8, 8, 8));

    let fcvtzs = decode(0x65D8_A020).unwrap(); // fcvtzs z0.s, p0/m, z1.d
    assert_eq!((fcvtzs.rd, fcvtzs.rn, fcvtzs.cond), (0, 1, 0));
    assert_eq!((fcvtzs.size, fcvtzs.imm, fcvtzs.rm), (8, 8, 4));
}
