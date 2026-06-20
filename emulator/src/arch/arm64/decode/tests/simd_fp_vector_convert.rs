use super::*;

#[test]
fn decode_simd_fcvtas_vector_forms() {
    assert_decode_cases(&[
        (0x4E21_C802, Opcode::SimdFcvtas, "fcvtas"),
        (0x4E61_CBBB, Opcode::SimdFcvtas, "fcvtas"),
    ]);

    let singles = decode(0x4E21_C802).unwrap();
    assert_eq!(singles.rd, 2);
    assert_eq!(singles.rn, 0);
    assert_eq!(singles.imm, 4);
    assert_eq!(singles.size, 16);

    let doubles = decode(0x4E61_CBBB).unwrap();
    assert_eq!(doubles.rd, 27);
    assert_eq!(doubles.rn, 29);
    assert_eq!(doubles.imm, 8);
    assert_eq!(doubles.size, 16);

    assert!(decode(0x0E61_C800).is_none());
}

#[test]
fn decode_simd_fp_long_narrow_forms() {
    let cases = [
        (0x0E21_7800, Opcode::SimdFcvtl, 2, 4, 16, "fcvtl"),
        (0x4E21_7800, Opcode::SimdFcvtl2, 2, 4, 16, "fcvtl2"),
        (0x0E61_7BA4, Opcode::SimdFcvtl, 4, 8, 16, "fcvtl"),
        (0x4E61_7BA7, Opcode::SimdFcvtl2, 4, 8, 16, "fcvtl2"),
        (0x0E21_6800, Opcode::SimdFcvtn, 4, 2, 8, "fcvtn"),
        (0x4E21_6800, Opcode::SimdFcvtn2, 4, 2, 8, "fcvtn2"),
        (0x0E61_6842, Opcode::SimdFcvtn, 8, 4, 8, "fcvtn"),
        (0x4E61_6BC2, Opcode::SimdFcvtn2, 8, 4, 8, "fcvtn2"),
    ];

    for (raw, op, src_size, dst_size, vector_size, mnemonic) in cases {
        assert_disarm64_mnemonic(raw, mnemonic);
        let instr = decode(raw).unwrap();
        assert_eq!(instr.op, op);
        assert_eq!(
            (instr.imm, instr.cond, instr.size),
            (src_size, dst_size, vector_size)
        );
    }
}
