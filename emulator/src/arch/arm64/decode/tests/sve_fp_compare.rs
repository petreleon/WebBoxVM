use super::*;

#[test]
fn decode_sve_fp_absolute_compare_forms() {
    let cases = [
        (0x6541_C010, Opcode::SveFpFacge, "facge"),
        (0x6541_E010, Opcode::SveFpFacgt, "facgt"),
        (0x6580_C772, Opcode::SveFpFacge, "facge"),
        (0x65DB_FFD3, Opcode::SveFpFacgt, "facgt"),
        (0x65C0_C372, Opcode::SveFpFacge, "facge"),
        (0x659F_E012, Opcode::SveFpFacgt, "facgt"),
        (0x6541_6000, Opcode::SveFpFcmeq, "fcmeq"),
        (0x6584_4861, Opcode::SveFpFcmge, "fcmge"),
        (0x65C6_4CB2, Opcode::SveFpFcmgt, "fcmgt"),
        (0x659F_7FD3, Opcode::SveFpFcmne, "fcmne"),
        (0x65D0_2901, Opcode::SveFpFcmge, "fcmge"),
        (0x6590_2D32, Opcode::SveFpFcmgt, "fcmgt"),
        (0x6552_3143, Opcode::SveFpFcmeq, "fcmeq"),
        (0x6593_3564, Opcode::SveFpFcmne, "fcmne"),
        (0x65D1_3985, Opcode::SveFpFcmlt, "fcmlt"),
        (0x6591_3DB6, Opcode::SveFpFcmle, "fcmle"),
    ];

    for (raw, expected, mnemonic) in cases {
        assert_disarm64_mnemonic(raw, mnemonic);
        assert_eq!(decode(raw).unwrap().op, expected, "raw=0x{raw:08x}");
    }

    let ge = decode(0x6580_C772).unwrap(); // facge p2.s, p1/z, z27.s, z0.s
    assert_eq!(ge.rd, 2);
    assert_eq!(ge.rn, 27);
    assert_eq!(ge.rm, 0);
    assert_eq!(ge.cond, 1);
    assert_eq!(ge.size, 4);

    let gt = decode(0x65DB_FFD3).unwrap(); // facgt p3.d, p7/z, z30.d, z27.d
    assert_eq!(gt.rd, 3);
    assert_eq!(gt.rn, 30);
    assert_eq!(gt.rm, 27);
    assert_eq!(gt.cond, 7);
    assert_eq!(gt.size, 8);

    let zero = decode(0x65D0_2901).unwrap(); // fcmge p1.d, p2/z, z8.d, #0.0
    assert_eq!(zero.rd, 1);
    assert_eq!(zero.rn, 8);
    assert_eq!(zero.cond, 2);
    assert_eq!(zero.imm, 1);

    assert!(decode(0x6500_C010).is_none());
    assert!(decode(0x6500_6000).is_none());
}
