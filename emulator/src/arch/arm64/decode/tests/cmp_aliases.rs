use super::*;

#[test]
fn decode_shifted_register_cmp_aliases_cross_checked_with_disarm64() {
    let cases = [
        (0xEB00_003F, 0, 0, true),
        (0x6B00_00BF, 0, 0, false),
        (0xEB41_47FF, 1, 17, true),
    ];

    for (raw, cond, imm, sf) in cases {
        assert_disarm64_mnemonic(raw, "subs");
        let instr = decode(raw).unwrap();
        assert_eq!(instr.op, Opcode::Cmp, "raw=0x{raw:08x}");
        assert_eq!(instr.rd, 31, "raw=0x{raw:08x}");
        assert_eq!(instr.cond, cond, "raw=0x{raw:08x}");
        assert_eq!(instr.imm, imm, "raw=0x{raw:08x}");
        assert_eq!(instr.sf, sf, "raw=0x{raw:08x}");
    }
}
