use super::*;

#[test]
fn decode_sve_fp_div_and_reverse_forms() {
    let cases = [
        (0x654D_8020, Opcode::SveFpDiv, "fdiv"),
        (0x658D_8020, Opcode::SveFpDiv, "fdiv"),
        (0x65CD_8C82, Opcode::SveFpDiv, "fdiv"),
        (0x658C_98E5, Opcode::SveFpDivr, "fdivr"),
        (0x65CC_8528, Opcode::SveFpDivr, "fdivr"),
        (0x65CD_83DF, Opcode::SveFpDiv, "fdiv"),
        (0x65CC_801F, Opcode::SveFpDivr, "fdivr"),
    ];

    for (raw, expected, mnemonic) in cases {
        assert_disarm64_mnemonic(raw, mnemonic);
        assert_eq!(decode(raw).unwrap().op, expected, "raw=0x{raw:08x}");
    }

    let div = decode(0x65CD_8C82).unwrap(); // fdiv z2.d, p3/m, z2.d, z4.d
    assert_eq!(div.rd, 2);
    assert_eq!(div.rm, 4);
    assert_eq!(div.cond, 3);
    assert_eq!(div.size, 8);

    let divr = decode(0x658C_98E5).unwrap(); // fdivr z5.s, p6/m, z5.s, z7.s
    assert_eq!(divr.rd, 5);
    assert_eq!(divr.rm, 7);
    assert_eq!(divr.cond, 6);
    assert_eq!(divr.size, 4);

    assert!(decode(0x650D_8000).is_none());
}
