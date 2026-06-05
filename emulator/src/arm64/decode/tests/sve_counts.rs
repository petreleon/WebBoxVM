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
}
