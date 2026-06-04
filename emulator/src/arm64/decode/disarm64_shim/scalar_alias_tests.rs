use super::*;

#[test]
fn maps_scalar_alias_mnemonics() {
    let cases = [
        (0xAA01_03E0, Opcode::MovReg, "orr"),
        (0x2A01_03E0, Opcode::MovReg, "orr"),
        (0x9340_7C62, Opcode::Sxtw, "sbfm"),
        (0x1381_0820, Opcode::Extr, "extr"),
        (0x1384_1C62, Opcode::Extr, "extr"),
    ];

    for (raw, expected, mnemonic) in cases {
        let decoded = decoder::decode(raw).expect("disarm64 should decode scalar alias word");
        assert_eq!(format!("{:?}", decoded.mnemonic), mnemonic);
        assert_eq!(mnemonic_to_opcode(raw, decoded.mnemonic), Some(expected));
    }
}

#[test]
fn leaves_non_alias_orr_and_sbfm_on_core_opcodes() {
    for (raw, expected, mnemonic) in [
        (0xAA01_0000, Opcode::OrrReg, "orr"),
        (0x9343_3020, Opcode::Sbfm, "sbfm"),
    ] {
        let decoded = decoder::decode(raw).expect("disarm64 should decode non-alias word");
        assert_eq!(format!("{:?}", decoded.mnemonic), mnemonic);
        assert_eq!(mnemonic_to_opcode(raw, decoded.mnemonic), Some(expected));
    }
}
