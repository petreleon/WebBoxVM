use super::*;

#[test]
fn decode_chk_gcs_and_sme_stop_aliases() {
    let cases = [
        (0xD503_251F, Opcode::Chkfeat, "chkfeat"),
        (0xD50B_7705, Opcode::GcsPushM, "gcspushm"),
        (0xD508_779F, Opcode::GcsPushX, "gcspushx"),
        (0xD52B_7725, Opcode::GcsPopM, "gcspopm"),
        (0xD508_77DF, Opcode::GcsPopX, "gcspopx"),
        (0xD508_77BF, Opcode::GcsPopCx, "gcspopcx"),
        (0xD50B_7744, Opcode::GcsSs1, "gcsss1"),
        (0xD52B_777F, Opcode::GcsSs2, "gcsss2"),
        (0xD503_427F, Opcode::Smstop, "smstop"),
        (0xD503_447F, Opcode::Smstop, "smstop"),
        (0xD503_467F, Opcode::Smstop, "smstop"),
        (0xD50B_7462, Opcode::DcGva, "sys"),
        (0xD50B_7482, Opcode::DcGzva, "sys"),
    ];
    assert_decode_cases(&cases);

    let chk = decode(0xD503_251F).unwrap();
    assert_eq!(chk.rd, 16);

    let gcsss1 = decode(0xD50B_7744).unwrap();
    assert_eq!(gcsss1.rd, 4);

    let gcspopm = decode(0xD52B_7725).unwrap();
    assert_eq!(gcspopm.rd, 5);

    assert_eq!(decode(0xD50B_7462).unwrap().rd, 2);
    assert_eq!(decode(0xD50B_7482).unwrap().rd, 2);
}

#[test]
fn decode_sysl_system_instruction() {
    assert_disarm64_mnemonic(0xD528_7423, "sysl");

    let instr = decode(0xD528_7423).unwrap();
    assert_eq!(instr.op, Opcode::Sysl);
    assert_eq!(instr.rd, 3);
    assert_eq!(instr.imm, 0x43A1);
}

#[test]
fn decode_128_bit_system_instruction_classes() {
    let cases = [
        (0xD548_0000, Opcode::Sysp, "sysp"),
        (0xD570_0000, Opcode::Mrrs, "mrrs"),
        (0xD550_0000, Opcode::Msrr, "msrr"),
    ];
    assert_decode_cases(&cases);

    let sysp = decode(0xD548_0020).unwrap();
    assert_eq!(sysp.op, Opcode::Sysp);
    assert_eq!(sysp.rd, 0);
    assert_eq!(sysp.imm, 0x4001);

    assert_eq!(decode(0xD52B_7725).unwrap().op, Opcode::GcsPopM);
}

#[test]
fn decode_pauth_hint_aliases_as_named_noops() {
    let cases = [
        (0xD503_211F, Opcode::Pacia1716, "pacia1716"),
        (0xD503_215F, Opcode::Pacib1716, "pacib1716"),
        (0xD503_219F, Opcode::Autia1716, "autia1716"),
        (0xD503_21DF, Opcode::Autib1716, "autib1716"),
        (0xD503_231F, Opcode::Paciaz, "paciaz"),
        (0xD503_233F, Opcode::Paciasp, "paciasp"),
        (0xD503_235F, Opcode::Pacibz, "pacibz"),
        (0xD503_237F, Opcode::Pacibsp, "pacibsp"),
        (0xD503_239F, Opcode::Autiaz, "autiaz"),
        (0xD503_23BF, Opcode::Autiasp, "autiasp"),
        (0xD503_23DF, Opcode::Autibz, "autibz"),
        (0xD503_23FF, Opcode::Autibsp, "autibsp"),
        (0xD503_20FF, Opcode::Xpaclri, "xpaclri"),
    ];
    for (raw, expected, display) in cases {
        let decoded = disarm64::decoder::decode(raw).unwrap();
        assert_eq!(decoded.to_string(), display);
        assert_eq!(decode(raw).unwrap().op, expected, "raw=0x{raw:08x}");
    }
}

#[test]
fn decode_bti_hint_aliases_as_named_noops() {
    let cases = [
        (0xD503_241F, Opcode::Bti, "bti"),
        (0xD503_245F, Opcode::BtiC, "bti\t\tc"),
        (0xD503_249F, Opcode::BtiJ, "bti\t\tj"),
        (0xD503_24DF, Opcode::BtiJc, "bti\t\tjc"),
    ];
    for (raw, expected, display) in cases {
        let decoded = disarm64::decoder::decode(raw).unwrap();
        assert_eq!(decoded.to_string(), display);
        assert_eq!(decode(raw).unwrap().op, expected, "raw=0x{raw:08x}");
    }
}
