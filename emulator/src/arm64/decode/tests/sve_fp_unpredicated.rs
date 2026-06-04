use super::*;

#[test]
fn decode_sve_fp_unpredicated_add_sub_forms_cross_checked_with_disarm64() {
    assert_decode_cases(&[
        (0x655D_003D, Opcode::SveFpAdd, "fadd"),
        (0x659E_03FE, Opcode::SveFpAdd, "fadd"),
        (0x655D_043D, Opcode::SveFpSub, "fsub"),
        (0x659E_07FE, Opcode::SveFpSub, "fsub"),
    ]);

    let fadd = decode(0x659E_03FE).unwrap(); // fadd z30.s, z31.s, z30.s
    assert_eq!((fadd.rd, fadd.rn, fadd.rm), (30, 31, 30));
    assert_eq!((fadd.cond, fadd.size), (0xFF, 4));

    let fsub_h = decode(0x655D_043D).unwrap(); // fsub z29.h, z1.h, z29.h
    assert_eq!((fsub_h.rd, fsub_h.rn, fsub_h.rm), (29, 1, 29));
    assert_eq!((fsub_h.cond, fsub_h.size), (0xFF, 2));
}
