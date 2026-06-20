use super::*;

#[test]
fn decode_sm3_crypto_forms_cross_checked_with_disarm64() {
    let cases = [
        (0xCE54_18B7, Opcode::SimdSm3Ss1, "sm3ss1", 23, 5, 20, 6, 0),
        (
            0xCE76_C6E4,
            Opcode::SimdSm3Partw2,
            "sm3partw2",
            4,
            23,
            22,
            0,
            0,
        ),
        (0xCE56_82E5, Opcode::SimdSm3Tt1A, "sm3tt1a", 5, 23, 22, 0, 0),
        (0xCE56_96E5, Opcode::SimdSm3Tt1B, "sm3tt1b", 5, 23, 22, 0, 1),
        (0xCE40_AAE6, Opcode::SimdSm3Tt2A, "sm3tt2a", 6, 23, 0, 0, 2),
        (0xCE43_BEE6, Opcode::SimdSm3Tt2B, "sm3tt2b", 6, 23, 3, 0, 3),
    ];

    for (raw, op, mnemonic, rd, rn, rm, cond, imm) in cases {
        assert_disarm64_mnemonic(raw, mnemonic);
        let decoded = decode(raw).unwrap();
        assert_eq!(decoded.op, op, "raw=0x{raw:08x}");
        assert_eq!((decoded.rd, decoded.rn, decoded.rm), (rd, rn, rm));
        assert_eq!((decoded.cond, decoded.imm), (cond, imm));
        assert_eq!(decoded.size, 16);
    }
}
