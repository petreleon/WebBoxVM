use super::*;

#[test]
fn decode_simd_scalar_bitwise_compare_forms_cross_checked_with_disarm64() {
    let cases = [
        (0x5EE0_8C20, Opcode::SimdCmtst, "cmtst"),
        (0x7EE0_8C20, Opcode::SimdCmeqReg, "cmeq"),
    ];
    assert_decode_cases(&cases);

    let cmtst = decode(0x5EE0_8C20).unwrap();
    assert_eq!((cmtst.rd, cmtst.rn, cmtst.rm), (0, 1, 0));
    assert_eq!((cmtst.imm, cmtst.size), (8, 8));
    let cmeq = decode(0x7EE0_8C20).unwrap();
    assert_eq!((cmeq.rd, cmeq.rn, cmeq.rm), (0, 1, 0));
    assert_eq!((cmeq.imm, cmeq.size), (8, 8));
    assert!(decode(0x5EA0_8C20).is_none());
    assert!(decode(0x7EA0_8C20).is_none());
}
