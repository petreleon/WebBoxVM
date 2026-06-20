use super::*;

#[test]
fn decode_shared_simd_mnemonics_cross_checked_with_disarm64() {
    let cases = [
        (0x6E3E_1FFE, Opcode::SimdEor, "eor", 16),
        (0x6E3F_1FBD, Opcode::SimdEor, "eor", 16),
        (0x2E20_1C00, Opcode::SimdEor, "eor", 8),
        (0x6F00_041F, Opcode::SimdMovi, "mvni", 16),
        (0x2F03_D7FE, Opcode::SimdMvni, "mvni", 8),
    ];

    for (raw, expected, mnemonic, size) in cases {
        assert_disarm64_mnemonic(raw, mnemonic);
        let instr = decode(raw).unwrap();
        assert_eq!(instr.op, expected, "raw=0x{raw:08x}");
        assert_eq!(instr.size, size, "raw=0x{raw:08x}");
    }
}
