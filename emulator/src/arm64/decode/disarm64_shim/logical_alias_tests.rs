use super::*;

#[test]
fn maps_scalar_negated_logical_mnemonics() {
    let cases = [
        (0x8A22_0020, Opcode::AndReg, "bic"),
        (0xAA25_0083, Opcode::OrrReg, "orn"),
        (0xCA28_00E6, Opcode::EorReg, "eon"),
        (0xEA2B_0149, Opcode::AndsReg, "bics"),
        (0x0A2E_0DAC, Opcode::AndReg, "bic"),
        (0x2AB1_120F, Opcode::OrrReg, "orn"),
        (0x4AF4_1672, Opcode::EorReg, "eon"),
    ];

    for (raw, expected, mnemonic) in cases {
        let decoded = decoder::decode(raw).expect("disarm64 should decode logical word");
        assert_eq!(format!("{:?}", decoded.mnemonic), mnemonic);
        assert_eq!(mnemonic_to_opcode(raw, decoded.mnemonic), Some(expected));
    }
}
