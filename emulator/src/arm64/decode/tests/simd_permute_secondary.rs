use super::*;

#[test]
fn decode_simd_secondary_permute_forms_cross_checked_with_disarm64() {
    let cases = [
        (0x4E9C_5BDE, Opcode::SimdUzp2, "uzp2", 4, 16),
        (0x4E5B_5BBD, Opcode::SimdUzp2, "uzp2", 2, 16),
        (0x4E0C_690B, Opcode::SimdTrn2, "trn2", 1, 16),
        (0x4ECE_698B, Opcode::SimdTrn2, "trn2", 8, 16),
    ];

    for (raw, expected, mnemonic, element_size, vector_size) in cases {
        assert_disarm64_mnemonic(raw, mnemonic);
        let instr = decode(raw).unwrap();
        assert_eq!(instr.op, expected, "raw=0x{raw:08x}");
        assert_eq!(instr.imm, element_size, "raw=0x{raw:08x}");
        assert_eq!(instr.size, vector_size, "raw=0x{raw:08x}");
    }
}
