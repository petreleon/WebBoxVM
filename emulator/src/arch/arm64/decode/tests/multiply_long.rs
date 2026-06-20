use super::*;

#[test]
fn decode_widening_multiply_forms_cross_checked_with_disarm64() {
    let cases = [
        (0x9B22_0C20, Opcode::Madd, "smaddl", 2, 3),
        (0x9BA6_1CA4, Opcode::Madd, "umaddl", 1, 7),
        (0x9B2A_AD28, Opcode::Msub, "smsubl", 2, 11),
        (0x9BAE_BDAC, Opcode::Msub, "umsubl", 1, 15),
        (0x9B32_7E30, Opcode::Madd, "smaddl", 2, 31),
        (0x9BB5_7E93, Opcode::Madd, "umaddl", 1, 31),
        (0x9B38_FEF6, Opcode::Msub, "smsubl", 2, 31),
        (0x9BBB_FF59, Opcode::Msub, "umsubl", 1, 31),
    ];

    for (raw, expected, mnemonic, size, ra) in cases {
        assert_disarm64_mnemonic(raw, mnemonic);
        let instr = decode(raw).unwrap();
        assert_eq!(instr.op, expected, "raw=0x{raw:08x}");
        assert!(instr.sf, "raw=0x{raw:08x}");
        assert_eq!(instr.size, size, "raw=0x{raw:08x}");
        assert_eq!(instr.cond, ra, "raw=0x{raw:08x}");
    }
}
