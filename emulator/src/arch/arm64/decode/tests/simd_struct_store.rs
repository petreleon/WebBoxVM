use super::*;

#[test]
fn decode_simd_structured_store_forms_cross_checked_with_disarm64() {
    let cases = [
        (0x4C00_803E, Opcode::SimdSt2, "st4", 30, 1, 0xFF, 0, 16),
        (0x4C9F_843C, Opcode::SimdSt2, "st4", 28, 1, 0xFE, 1, 16),
        (0x0C00_443E, Opcode::SimdSt3, "st4", 30, 1, 0xFF, 1, 8),
        (0x4C00_043A, Opcode::SimdSt4, "st4", 26, 1, 0xFF, 1, 16),
        (0x4C9F_0436, Opcode::SimdSt4, "st4", 22, 1, 0xFE, 1, 16),
        (0x0DBF_A013, Opcode::SimdSt4Single, "st4", 19, 0, 0xFE, 4, 4),
    ];

    for (raw, op, mnemonic, rd, rn, rm, cond, size) in cases {
        assert_disarm64_mnemonic(raw, mnemonic);
        let decoded = decode(raw).unwrap();
        assert_eq!(decoded.op, op, "raw=0x{raw:08x}");
        assert_eq!(decoded.rd, rd);
        assert_eq!(decoded.rn, rn);
        assert_eq!(decoded.rm, rm);
        assert_eq!(decoded.cond, cond);
        assert_eq!(decoded.size, size);
    }
}
