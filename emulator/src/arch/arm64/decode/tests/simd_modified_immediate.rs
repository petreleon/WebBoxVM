use super::*;

#[test]
fn decode_simd_modified_immediates_cross_checked_with_disarm64() {
    let cases = [
        (0x4F04_741F, Opcode::SimdOrrImm, "orr", 0x8000_0000, 4, 16),
        (0x0F04_741F, Opcode::SimdOrrImm, "orr", 0x8000_0000, 4, 8),
        (0x0F00_943F, Opcode::SimdOrrImm, "orr", 0x0001, 2, 8),
        (0x4F03_D7FE, Opcode::SimdMovi, "movi", 0x007f_ffff, 4, 16),
        (0x4F07_C7F7, Opcode::SimdMovi, "movi", 0x0000_ffff, 4, 16),
    ];

    for (raw, expected, mnemonic, imm, cond, size) in cases {
        assert_disarm64_mnemonic(raw, mnemonic);
        let instr = decode(raw).unwrap();
        assert_eq!(instr.op, expected, "raw=0x{raw:08x}");
        assert_eq!(instr.imm, imm, "raw=0x{raw:08x}");
        assert_eq!(instr.cond, cond, "raw=0x{raw:08x}");
        assert_eq!(instr.size, size, "raw=0x{raw:08x}");
    }
}
