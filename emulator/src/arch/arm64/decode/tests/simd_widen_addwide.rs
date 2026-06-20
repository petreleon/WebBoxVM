use super::*;

#[test]
fn decode_simd_add_wide_forms_cross_checked_with_disarm64() {
    let cases = [
        (
            0x0E7E_13BD,
            Opcode::SimdSaddw,
            "saddw",
            29,
            29,
            30,
            2,
            false,
        ),
        (
            0x4E7E_13FF,
            Opcode::SimdSaddw,
            "saddw2",
            31,
            31,
            30,
            2,
            true,
        ),
        (
            0x2EBF_13DF,
            Opcode::SimdUaddw,
            "uaddw",
            31,
            30,
            31,
            4,
            false,
        ),
        (0x6EA8_10E6, Opcode::SimdUaddw, "uaddw2", 6, 7, 8, 4, true),
        (0x2E22_0230, Opcode::SimdUaddl, "uaddl", 16, 17, 2, 1, false),
        (0x6E22_0231, Opcode::SimdUaddl, "uaddl2", 17, 17, 2, 1, true),
        (0x0E23_1041, Opcode::SimdSaddw, "saddw", 1, 2, 3, 1, false),
        (0x2E26_10A4, Opcode::SimdUaddw, "uaddw", 4, 5, 6, 1, false),
        (0x2E23_32D6, Opcode::SimdUsubw, "usubw", 22, 22, 3, 1, false),
        (0x6E23_32F7, Opcode::SimdUsubw, "usubw2", 23, 23, 3, 1, true),
    ];

    for (raw, op, mnemonic, rd, rn, rm, element_size, upper_half) in cases {
        assert_disarm64_mnemonic(raw, mnemonic);
        let instr = decode(raw).unwrap();
        assert_eq!(instr.op, op);
        assert_eq!(instr.rd, rd);
        assert_eq!(instr.rn, rn);
        assert_eq!(instr.rm, rm);
        assert_eq!(instr.cond, element_size);
        assert_eq!(instr.sf, upper_half);
        assert_eq!(instr.size, 16);
    }
}
