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
