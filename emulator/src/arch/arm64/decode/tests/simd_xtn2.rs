use super::*;

#[test]
fn decode_simd_xtn2_cross_checked_with_disarm64() {
    let cases = [
        (0x4EA1_2BEF, Opcode::SimdXtn2, "xtn2", 4),
        (0x4E21_2800, Opcode::SimdXtn2, "xtn2", 1),
    ];

    for (raw, expected, mnemonic, element_size) in cases {
        assert_disarm64_mnemonic(raw, mnemonic);
        let instr = decode(raw).unwrap();
        assert_eq!(instr.op, expected, "raw=0x{raw:08x}");
        assert_eq!(instr.imm, element_size, "raw=0x{raw:08x}");
        assert_eq!(instr.size, 16, "raw=0x{raw:08x}");
    }

    assert!(decode(0x4EE1_2800).is_none());
}
