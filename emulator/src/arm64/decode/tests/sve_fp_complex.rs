use super::*;

#[test]
fn decode_sve_fcmla_vectors_cross_checked_with_disarm64() {
    let cases = [
        (0x6404_4626, Opcode::SveFpFcmla, "fcmla"),
        (0x6444_0626, Opcode::SveFpFcmla, "fcmla"),
        (0x6484_6626, Opcode::SveFpFcmla, "fcmla"),
    ];
    assert_decode_cases(&cases);

    let half = decode(0x6404_4626).unwrap(); // fcmla z6.h, p1/m, z17.h, z4.h, #180
    assert_eq!(half.rd, 6);
    assert_eq!(half.rn, 17);
    assert_eq!(half.rm, 4);
    assert_eq!(half.cond, 1);
    assert_eq!(half.imm, 180);
    assert_eq!(half.size, 2);

    let single = decode(0x6444_2626).unwrap(); // fcmla z6.s, p1/m, z17.s, z4.s, #90
    assert_eq!(single.imm, 90);
    assert_eq!(single.size, 4);

    let double = decode(0x6484_6626).unwrap(); // fcmla z6.d, p1/m, z17.d, z4.d, #270
    assert_eq!(double.imm, 270);
    assert_eq!(double.size, 8);
}
