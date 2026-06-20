use super::*;

#[test]
fn decode_scalar_fp_minmax_forms_cross_checked_with_disarm64() {
    let cases = [
        (0x1E22_4820, Opcode::FpMax, "fmax"),
        (0x1E25_5883, Opcode::FpMin, "fmin"),
        (0x1E68_48E6, Opcode::FpMax, "fmax"),
        (0x1E6B_5949, Opcode::FpMin, "fmin"),
    ];
    assert_decode_cases(&cases);

    let fmax_s = decode(0x1E22_4820).unwrap();
    assert_eq!((fmax_s.rd, fmax_s.rn, fmax_s.rm), (0, 1, 2));
    assert_eq!(fmax_s.size, 4);

    let fmin_d = decode(0x1E6B_5949).unwrap();
    assert_eq!((fmin_d.rd, fmin_d.rn, fmin_d.rm), (9, 10, 11));
    assert_eq!(fmin_d.size, 8);
}
