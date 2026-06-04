use super::*;

#[test]
fn decode_simd_ucvtf_forms_cross_checked_with_disarm64() {
    let cases = [
        (0x7E61_DBDE, 30, 30, 8, 8),
        (0x7E21_D821, 1, 1, 4, 4),
        (0x6E61_D800, 0, 0, 8, 16),
        (0x6E21_D821, 1, 1, 4, 16),
        (0x2E21_D842, 2, 2, 4, 8),
    ];

    for (raw, rd, rn, element_size, vector_size) in cases {
        assert_disarm64_mnemonic(raw, "ucvtf");
        let instr = decode(raw).unwrap();
        assert_eq!(instr.op, Opcode::SimdUcvtf);
        assert_eq!(instr.rd, rd);
        assert_eq!(instr.rn, rn);
        assert_eq!(instr.imm, element_size);
        assert_eq!(instr.size, vector_size);
    }

    assert!(decode(0x0E61_D800).is_none());
}
