use super::*;

#[test]
fn decode_sve_scalar_vector_length_forms() {
    let cases = [
        (0x0420_E3E0, Opcode::SveCnt, "cntb"),
        (0x0460_E3E0, Opcode::SveCnt, "cnth"),
        (0x04A0_E3E0, Opcode::SveCnt, "cntw"),
        (0x04E0_E3E0, Opcode::SveCnt, "cntd"),
        (0x043F_57FF, Opcode::SveAddvl, "addvl"),
        (0x0430_5A10, Opcode::SveAddsvl, "addsvl"),
        (0x047F_57FF, Opcode::SveAddpl, "addpl"),
        (0x0470_5A10, Opcode::SveAddspl, "addspl"),
        (0x04BF_5210, Opcode::SveRdvl, "rdvl"),
        (0x04BF_5A10, Opcode::SveRdsvl, "rdsvl"),
        (0x0431_E3F1, Opcode::SveIncScalar, "incb"),
        (0x0471_E3F1, Opcode::SveIncScalar, "inch"),
        (0x04B1_E3F1, Opcode::SveIncScalar, "incw"),
        (0x04F1_E3F1, Opcode::SveIncScalar, "incd"),
        (0x0431_E7F1, Opcode::SveDecScalar, "decb"),
        (0x0471_E7F1, Opcode::SveDecScalar, "dech"),
        (0x04B1_E7F1, Opcode::SveDecScalar, "decw"),
        (0x04F1_E7F1, Opcode::SveDecScalar, "decd"),
        (0x252C_88E1, Opcode::SveIncpScalar, "incp"),
        (0x256C_88E1, Opcode::SveIncpScalar, "incp"),
        (0x25AC_88E1, Opcode::SveIncpScalar, "incp"),
        (0x25EC_88E1, Opcode::SveIncpScalar, "incp"),
        (0x252D_88E1, Opcode::SveDecpScalar, "decp"),
        (0x256D_88E1, Opcode::SveDecpScalar, "decp"),
        (0x25AD_88E1, Opcode::SveDecpScalar, "decp"),
        (0x25ED_88E1, Opcode::SveDecpScalar, "decp"),
        (0x256C_80E1, Opcode::SveIncpVector, "incp"),
        (0x25AC_80E1, Opcode::SveIncpVector, "incp"),
        (0x25EC_80E1, Opcode::SveIncpVector, "incp"),
        (0x256D_80E1, Opcode::SveDecpVector, "decp"),
        (0x25AD_80E1, Opcode::SveDecpVector, "decp"),
        (0x25ED_80E1, Opcode::SveDecpVector, "decp"),
    ];
    for (raw, expected, mnemonic) in cases {
        assert_disarm64_mnemonic(raw, mnemonic);
        assert_eq!(decode(raw).unwrap().op, expected, "raw=0x{raw:08x}");
    }

    let cntd = decode(0x04E1_E3E0).unwrap(); // cntd x0, all, mul #2
    assert_eq!(cntd.op, Opcode::SveCnt);
    assert_eq!(cntd.rd, 0);
    assert_eq!(cntd.cond, 31);
    assert_eq!(cntd.imm, 2);
    assert_eq!(cntd.size, 8);

    let cntd_vl1 = decode(0x04E0_E020).unwrap();
    assert_eq!(cntd_vl1.cond, 1);

    let addvl = decode(0x043F_57FF).unwrap(); // addvl sp, sp, #-1
    assert_eq!(addvl.rd, 31);
    assert_eq!(addvl.rn, 31);
    assert_eq!(addvl.imm as i64, -1);

    let addsvl = decode(0x0430_5A10).unwrap();
    assert_eq!(addsvl.rd, 16);
    assert_eq!(addsvl.rn, 16);
    assert_eq!(addsvl.imm as i64, 16);

    let addpl = decode(0x047F_57FF).unwrap(); // addpl sp, sp, #-1
    assert_eq!(addpl.rd, 31);
    assert_eq!(addpl.rn, 31);
    assert_eq!(addpl.imm as i64, -1);

    let addspl = decode(0x0470_5A10).unwrap();
    assert_eq!(addspl.rd, 16);
    assert_eq!(addspl.rn, 16);
    assert_eq!(addspl.imm as i64, 16);

    let rdsvl = decode(0x04BF_5A10).unwrap();
    assert_eq!(rdsvl.rd, 16);
    assert_eq!(rdsvl.imm as i64, 16);

    let incb = decode(0x0431_E3F1).unwrap(); // incb x17, all, mul #2
    assert_eq!(incb.rd, 17);
    assert_eq!(incb.rn, 17);
    assert_eq!(incb.cond, 31);
    assert_eq!(incb.imm, 2);
    assert_eq!(incb.size, 1);

    let decd = decode(0x04F1_E7F1).unwrap(); // decd x17, all, mul #2
    assert_eq!(decd.op, Opcode::SveDecScalar);
    assert_eq!(decd.size, 8);

    let incp = decode(0x25EC_88E1).unwrap(); // incp x1, p7.d
    assert_eq!(incp.rd, 1);
    assert_eq!(incp.rn, 1);
    assert_eq!(incp.cond, 7);
    assert_eq!(incp.size, 8);

    let decp = decode(0x252D_88E1).unwrap(); // decp x1, p7.b
    assert_eq!(decp.op, Opcode::SveDecpScalar);
    assert_eq!(decp.size, 1);

    let incp_vec = decode(0x25EC_80E1).unwrap(); // incp z1.d, p7.d
    assert_eq!(incp_vec.op, Opcode::SveIncpVector);
    assert_eq!(incp_vec.rd, 1);
    assert_eq!(incp_vec.cond, 7);
    assert_eq!(incp_vec.size, 8);
    assert!(decode(0x252C_80E1).is_none()); // vector .b form is reserved
}
