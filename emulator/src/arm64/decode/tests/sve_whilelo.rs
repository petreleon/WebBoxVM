use super::*;

#[test]
fn decode_sve_whilelo_forms_cross_checked_with_disarm64() {
    let cases = [
        (
            0x2522_0CE1,
            Opcode::SveWhileLo,
            "whilelo",
            1,
            7,
            2,
            1,
            false,
        ),
        (
            0x2522_00A1,
            Opcode::SveWhileGe,
            "whilege",
            1,
            5,
            2,
            1,
            false,
        ),
        (
            0x2522_00B1,
            Opcode::SveWhileGt,
            "whilegt",
            1,
            5,
            2,
            1,
            false,
        ),
        (
            0x2522_04A1,
            Opcode::SveWhileLt,
            "whilelt",
            1,
            5,
            2,
            1,
            false,
        ),
        (
            0x2522_04B1,
            Opcode::SveWhileLe,
            "whilele",
            1,
            5,
            2,
            1,
            false,
        ),
        (
            0x2522_08A1,
            Opcode::SveWhileHs,
            "whilehs",
            1,
            5,
            2,
            1,
            false,
        ),
        (
            0x2522_08B1,
            Opcode::SveWhileHi,
            "whilehi",
            1,
            5,
            2,
            1,
            false,
        ),
        (
            0x2522_0CB1,
            Opcode::SveWhileLs,
            "whilels",
            1,
            5,
            2,
            1,
            false,
        ),
        (0x2522_1CE1, Opcode::SveWhileLo, "whilelo", 1, 7, 2, 1, true),
        (
            0x2562_0FE0,
            Opcode::SveWhileLo,
            "whilelo",
            0,
            31,
            2,
            2,
            false,
        ),
        (
            0x25A6_0FE1,
            Opcode::SveWhileLo,
            "whilelo",
            1,
            31,
            6,
            4,
            false,
        ),
        (0x25E9_1CA3, Opcode::SveWhileLo, "whilelo", 3, 5, 9, 8, true),
        (0x25E9_10A3, Opcode::SveWhileGe, "whilege", 3, 5, 9, 8, true),
    ];

    for (raw, op, mnemonic, rd, rn, rm, element_size, wide) in cases {
        assert_disarm64_mnemonic(raw, mnemonic);
        let instr = decode(raw).unwrap();
        assert_eq!(instr.op, op);
        assert_eq!(instr.rd, rd);
        assert_eq!(instr.rn, rn);
        assert_eq!(instr.rm, rm);
        assert_eq!(instr.size, element_size);
        assert_eq!(instr.sf, wide);
    }
}
