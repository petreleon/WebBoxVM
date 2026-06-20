use super::*;

#[test]
fn decode_scalar_negated_logicals_cross_checked_with_disarm64() {
    let cases = [
        (0x8A22_0020, Opcode::AndReg, "bic", 4, 0),
        (0xAA25_0083, Opcode::OrrReg, "orn", 4, 0),
        (0xCA28_00E6, Opcode::EorReg, "eon", 4, 0),
        (0xEA2B_0149, Opcode::AndsReg, "bics", 4, 0),
        (0x0A2E_0DAC, Opcode::AndReg, "bic", 4, 3),
        (0x2AB1_120F, Opcode::OrrReg, "orn", 6, 4),
        (0x4AF4_1672, Opcode::EorReg, "eon", 7, 5),
    ];

    for (raw, expected, mnemonic, cond, imm) in cases {
        assert_disarm64_mnemonic(raw, mnemonic);
        let instr = decode(raw).unwrap();
        assert_eq!(instr.op, expected, "raw=0x{raw:08x}");
        assert_eq!(instr.cond, cond, "raw=0x{raw:08x}");
        assert_eq!(instr.imm, imm, "raw=0x{raw:08x}");
    }
}
