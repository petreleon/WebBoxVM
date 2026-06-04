use super::*;

#[test]
fn decode_sve_z_vector_forms() {
    let cases = [
        (0x0420_BF9A, Opcode::SveMovprfx, "movprfx"),
        (0x04D0_2C3D, Opcode::SveMovprfx, "movprfx"),
        (0x05E0_383E, Opcode::SveDupGpr, "dup"),
        (0x04F8_03D8, Opcode::SveAddVec, "add"),
        (0x04FE_07FE, Opcode::SveSubVec, "sub"),
        (0x0460_301B, Opcode::SveOrrVec, "orr"),
        (0x04A0_3000, Opcode::SveEorVec, "eor"),
        (0x05FF_C040, Opcode::SveSel, "sel"),
        (0x6580_8020, Opcode::SveFpAdd, "fadd"),
        (0x6581_8020, Opcode::SveFpSub, "fsub"),
        (0x65C2_8020, Opcode::SveFpMul, "fmul"),
        (0x65C3_8020, Opcode::SveFpSubr, "fsubr"),
        (0x65E7_039A, Opcode::SveFpFmla, "fmla"),
        (0x65E7_239A, Opcode::SveFpFmls, "fmls"),
        (0x65FC_83BF, Opcode::SveFpFmad, "fmad"),
        (0x65FC_A3E6, Opcode::SveFpFmsb, "fmsb"),
    ];
    for (raw, expected, mnemonic) in cases {
        assert_disarm64_mnemonic(raw, mnemonic);
        assert_eq!(decode(raw).unwrap().op, expected, "raw=0x{raw:08x}");
    }

    let mov = decode(0x0420_BF9A).unwrap(); // movprfx z26, z28
    assert_eq!(mov.rd, 26);
    assert_eq!(mov.rn, 28);
    assert_eq!(mov.cond, 0xFF);

    let pred_mov = decode(0x04D0_2C3D).unwrap(); // movprfx z29.d, p3/z, z1.d
    assert_eq!(pred_mov.rd, 29);
    assert_eq!(pred_mov.rn, 1);
    assert_eq!(pred_mov.cond, 3);
    assert_eq!(pred_mov.size, 8);
    assert!(!pred_mov.sf);

    let dup = decode(0x05E0_383E).unwrap(); // dup z30.d, x1
    assert_eq!(dup.rd, 30);
    assert_eq!(dup.rn, 1);
    assert_eq!(dup.size, 8);

    let add = decode(0x04F8_03D8).unwrap(); // add z24.d, z30.d, z24.d
    assert_eq!(add.rd, 24);
    assert_eq!(add.rn, 30);
    assert_eq!(add.rm, 24);
    assert_eq!(add.size, 8);

    let sel = decode(0x05FF_C040).unwrap(); // sel z0.d, p0, z2.d, z31.d
    assert_eq!(sel.rd, 0);
    assert_eq!(sel.rn, 2);
    assert_eq!(sel.rm, 31);
    assert_eq!(sel.cond, 0);

    let fmul = decode(0x65C2_84C5).unwrap(); // fmul z5.d, p1/m, z5.d, z6.d
    assert_eq!(fmul.op, Opcode::SveFpMul);
    assert_eq!(fmul.rd, 5);
    assert_eq!(fmul.rn, 5);
    assert_eq!(fmul.rm, 6);
    assert_eq!(fmul.cond, 1);
    assert_eq!(fmul.size, 8);

    let fmad = decode(0x65E9_8907).unwrap(); // fmad z7.d, p2/m, z8.d, z9.d
    assert_eq!(fmad.op, Opcode::SveFpFmad);
    assert_eq!(fmad.rd, 7);
    assert_eq!(fmad.rn, 7);
    assert_eq!(fmad.rm, 8);
    assert_eq!(fmad.imm, 9);
    assert_eq!(fmad.cond, 2);
    assert_eq!(fmad.size, 8);

    assert_ne!(
        decode(0x6583_BFFE).map(|instr| instr.op),
        Some(Opcode::SveFpSubr)
    );
    assert_ne!(
        decode(0x6580_A3DE).map(|instr| instr.op),
        Some(Opcode::SveFpAdd)
    );
}
