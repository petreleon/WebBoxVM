use super::*;

#[test]
fn decode_simd_rcpc_unscaled_load_store_forms() {
    let cases = [
        (0x1D40_0820, Opcode::SimdLdr, "ldapur", 1, false, 0, 1, 0),
        (0x5D40_1841, Opcode::SimdLdr, "ldapur", 2, false, 1, 2, 1),
        (0x9D40_2862, Opcode::SimdLdr, "ldapur", 4, false, 2, 3, 2),
        (0xDD40_3883, Opcode::SimdLdr, "ldapur", 8, true, 3, 4, 3),
        (0x1DC0_4884, Opcode::SimdLdr, "ldapur", 16, true, 4, 4, 4),
        (0x1D00_0820, Opcode::SimdStr, "stlur", 1, false, 0, 1, 0),
        (0x5D00_1841, Opcode::SimdStr, "stlur", 2, false, 1, 2, 1),
        (0x9D00_2862, Opcode::SimdStr, "stlur", 4, false, 2, 3, 2),
        (0xDD00_3883, Opcode::SimdStr, "stlur", 8, true, 3, 4, 3),
        (0x1D80_4884, Opcode::SimdStr, "stlur", 16, true, 4, 4, 4),
        (
            0x1D5F_F820,
            Opcode::SimdLdr,
            "ldapur",
            1,
            false,
            0,
            1,
            (-1i64) as u64,
        ),
    ];

    for (raw, expected, mnemonic, size, sf, rd, rn, imm) in cases {
        assert_disarm64_mnemonic(raw, mnemonic);
        let instr = decode(raw).unwrap();
        assert_eq!(instr.op, expected, "raw=0x{raw:08x}");
        assert_eq!(
            (instr.size, instr.sf, instr.rd, instr.rn, instr.imm),
            (size, sf, rd, rn, imm)
        );
    }
}
