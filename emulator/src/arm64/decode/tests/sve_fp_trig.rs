use super::*;

#[test]
fn decode_sve_ftmad_forms_cross_checked_with_disarm64() {
    assert_decode_cases(&[
        (0x6550_83C1, Opcode::SveFpFtmad, "ftmad"),
        (0x6594_83C1, Opcode::SveFpFtmad, "ftmad"),
        (0x65D7_83C1, Opcode::SveFpFtmad, "ftmad"),
    ]);

    let half = decode(0x6550_83C1).unwrap(); // ftmad z1.h, z1.h, z30.h, #0
    assert_eq!(
        (half.rd, half.rn, half.rm, half.imm, half.cond, half.size),
        (1, 1, 30, 0, 0xFF, 2)
    );

    let single = decode(0x6594_83C1).unwrap(); // ftmad z1.s, z1.s, z30.s, #4
    assert_eq!(
        (
            single.rd,
            single.rn,
            single.rm,
            single.imm,
            single.cond,
            single.size
        ),
        (1, 1, 30, 4, 0xFF, 4)
    );

    let double = decode(0x65D7_83C1).unwrap(); // ftmad z1.d, z1.d, z30.d, #7
    assert_eq!(
        (
            double.rd,
            double.rn,
            double.rm,
            double.imm,
            double.cond,
            double.size
        ),
        (1, 1, 30, 7, 0xFF, 8)
    );

    assert!(decode(0x6510_8000).is_none());
}
