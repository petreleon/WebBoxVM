use super::*;

#[test]
fn decode_simd_integer_compare_zero_forms_cross_checked_with_disarm64() {
    let cases = [
        (0x0E20_8800, Opcode::SimdCmgtZero, "cmgt"),
        (0x4EA0_8821, Opcode::SimdCmgtZero, "cmgt"),
        (0x5EE0_8823, Opcode::SimdCmgtZero, "cmgt"),
        (0x2E20_9800, Opcode::SimdCmleZero, "cmle"),
        (0x6EA0_9821, Opcode::SimdCmleZero, "cmle"),
        (0x7EE0_9823, Opcode::SimdCmleZero, "cmle"),
        (0x0E20_A800, Opcode::SimdCmltZero, "cmlt"),
        (0x4EA0_A821, Opcode::SimdCmltZero, "cmlt"),
        (0x5EE0_A823, Opcode::SimdCmltZero, "cmlt"),
    ];
    assert_decode_cases(&cases);

    let cmgt = decode(0x4EA0_8821).unwrap();
    assert_eq!((cmgt.rd, cmgt.rn, cmgt.imm, cmgt.size), (1, 1, 4, 16));
    let cmle = decode(0x7EE0_9823).unwrap();
    assert_eq!((cmle.rd, cmle.rn, cmle.imm, cmle.size), (3, 1, 8, 8));
    assert!(decode(0x0EE0_8800).is_none());
    assert!(decode(0x0EE0_A800).is_none());
    assert!(decode(0x2EE0_9800).is_none());
}
