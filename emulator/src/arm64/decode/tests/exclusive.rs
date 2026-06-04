use super::*;

#[test]
fn decode_exclusive_acquire_release_forms() {
    let cases = [
        (0xC85F_FC40, Opcode::Ldxr, "ldaxr", 0, 2, 31, 1, 8),
        (0x485F_FC83, Opcode::Ldxr, "ldaxrh", 3, 4, 31, 1, 2),
        (0x085F_FCC5, Opcode::Ldxr, "ldaxrb", 5, 6, 31, 1, 1),
        (0xC807_FD28, Opcode::Stxr, "stlxr", 8, 9, 31, 1, 8),
        (0x480A_FD8B, Opcode::Stxr, "stlxrh", 11, 12, 31, 1, 2),
        (0x080D_FDEE, Opcode::Stxr, "stlxrb", 14, 15, 31, 1, 1),
        (0x48DF_FE30, Opcode::Ldar, "ldarh", 16, 17, 31, 1, 2),
        (0x089F_FE72, Opcode::Stlr, "stlrb", 18, 19, 31, 1, 1),
    ];

    for (raw, expected, mnemonic, rd, rn, rm, cond, size) in cases {
        assert_disarm64_mnemonic(raw, mnemonic);
        let instr = decode(raw).unwrap();
        assert_eq!(instr.op, expected, "raw=0x{raw:08x}");
        assert_eq!(instr.rd, rd, "raw=0x{raw:08x}");
        assert_eq!(instr.rn, rn, "raw=0x{raw:08x}");
        assert_eq!(instr.rm, rm, "raw=0x{raw:08x}");
        assert_eq!(instr.cond, cond, "raw=0x{raw:08x}");
        assert_eq!(instr.size, size, "raw=0x{raw:08x}");
    }
}

#[test]
fn decode_exclusive_pair_acquire_release_forms() {
    let cases = [
        (0xC87F_8440, Opcode::Ldxp, "ldaxp", 0, 2, 1, 0, 1),
        (0xC823_8440, Opcode::Stxp, "stlxp", 0, 2, 1, 3, 1),
    ];

    for (raw, expected, mnemonic, rd, rn, rm, status, cond) in cases {
        assert_disarm64_mnemonic(raw, mnemonic);
        let instr = decode(raw).unwrap();
        assert_eq!(instr.op, expected, "raw=0x{raw:08x}");
        assert_eq!(instr.rd, rd, "raw=0x{raw:08x}");
        assert_eq!(instr.rn, rn, "raw=0x{raw:08x}");
        assert_eq!(instr.rm, rm, "raw=0x{raw:08x}");
        assert_eq!(instr.imm, status, "raw=0x{raw:08x}");
        assert_eq!(instr.cond, cond, "raw=0x{raw:08x}");
    }
}
