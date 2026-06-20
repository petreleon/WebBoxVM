use super::*;

#[test]
fn decode_rcpc3_pair_forms_cross_checked_with_disarm64() {
    let cases = [
        (0x9941_1840, Opcode::Ldp, "ldiapp", 0, 2, 1, false, 0, 0),
        (0x9944_08A3, Opcode::Ldp, "ldiapp", 3, 5, 4, false, 1, 8),
        (0xD947_1906, Opcode::Ldp, "ldiapp", 6, 8, 7, true, 0, 0),
        (0xD94A_0969, Opcode::Ldp, "ldiapp", 9, 11, 10, true, 1, 16),
        (0x990D_19CC, Opcode::Stp, "stilp", 12, 14, 13, false, 0, 0),
        (
            0x9910_0A2F,
            Opcode::Stp,
            "stilp",
            15,
            17,
            16,
            false,
            3,
            (-8i64) as u64,
        ),
        (0xD913_1A92, Opcode::Stp, "stilp", 18, 20, 19, true, 0, 0),
        (
            0xD916_0AF5,
            Opcode::Stp,
            "stilp",
            21,
            23,
            22,
            true,
            3,
            (-16i64) as u64,
        ),
    ];

    for (raw, expected, mnemonic, rd, rn, rm, sf, cond, imm) in cases {
        assert_disarm64_mnemonic(raw, mnemonic);
        let instr = decode(raw).unwrap();
        assert_eq!(instr.op, expected, "raw=0x{raw:08x}");
        assert_eq!((instr.rd, instr.rn, instr.rm), (rd, rn, rm));
        assert_eq!(
            (instr.sf, instr.cond, instr.imm, instr.size),
            (sf, cond, imm, 0)
        );
    }
}
