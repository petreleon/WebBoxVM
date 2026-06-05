use super::*;

#[test]
fn decode_sve_halfword_store_forms_cross_checked_with_disarm64() {
    let cases = [
        (0xE4A0_ECA7, 7, 5, 0xFF, 0, 3, 2),
        (0xE4C1_ECA7, 7, 5, 0xFF, 1, 3, 4),
        (0xE4AE_E082, 2, 4, 0xFF, -2, 0, 2),
        (0xE4FE_5C1F, 31, 0, 30, 0, 7, 8),
    ];

    for &(raw, rd, rn, rm, imm, pred, size) in &cases {
        assert_disarm64_mnemonic(raw, "st1h");
        let instr = decode(raw).unwrap();
        assert_eq!(instr.op, Opcode::SveSt1h);
        assert_eq!((instr.rd, instr.rn, instr.rm), (rd, rn, rm));
        assert_eq!(instr.imm as i64, imm);
        assert_eq!((instr.cond, instr.size), (pred, size));
    }
}

#[test]
fn decode_sve_halfword_scatter_store_forms_cross_checked_with_disarm64() {
    let cases = [
        (0xE4FF_8AB2, 18, 21, 31, 3, false, 2, 4),
        (0xE4FF_CAB2, 18, 21, 31, 3, true, 2, 4),
        (0xE4DF_8AB2, 18, 21, 31, 2, false, 2, 4),
        (0xE4DF_CAB2, 18, 21, 31, 2, true, 2, 4),
        (0xE4BF_8AB2, 18, 21, 31, 3, false, 2, 8),
        (0xE4BF_CAB2, 18, 21, 31, 3, true, 2, 8),
        (0xE49F_8AB2, 18, 21, 31, 2, false, 2, 8),
        (0xE49F_CAB2, 18, 21, 31, 2, true, 2, 8),
        (0xE4BF_AAA7, 7, 21, 31, 1, false, 2, 8),
        (0xE49F_AAA7, 7, 21, 31, 0, false, 2, 8),
    ];

    for &(raw, rd, rn, rm, imm, signed, pred, size) in &cases {
        assert_disarm64_mnemonic(raw, "st1h");
        let instr = decode(raw).unwrap();
        assert_eq!(instr.op, Opcode::SveSt1hScatter, "raw=0x{raw:08x}");
        assert_eq!((instr.rd, instr.rn, instr.rm), (rd, rn, rm));
        assert_eq!(
            (instr.imm, instr.sf, instr.cond, instr.size),
            (imm, signed, pred, size)
        );
    }
}
