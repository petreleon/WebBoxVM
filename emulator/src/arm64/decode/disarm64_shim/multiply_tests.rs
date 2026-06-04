use super::*;

#[test]
fn maps_widening_multiply_mnemonics() {
    let cases = [
        (0x9B22_0C20, Opcode::Madd, "smaddl"),
        (0x9BA6_1CA4, Opcode::Madd, "umaddl"),
        (0x9B2A_AD28, Opcode::Msub, "smsubl"),
        (0x9BAE_BDAC, Opcode::Msub, "umsubl"),
        (0x9B32_7E30, Opcode::Madd, "smaddl"),
        (0x9BB5_7E93, Opcode::Madd, "umaddl"),
        (0x9B38_FEF6, Opcode::Msub, "smsubl"),
        (0x9BBB_FF59, Opcode::Msub, "umsubl"),
    ];

    for (raw, expected, mnemonic) in cases {
        let decoded = decoder::decode(raw).expect("disarm64 should decode multiply word");
        assert_eq!(format!("{:?}", decoded.mnemonic), mnemonic);
        assert_eq!(mnemonic_to_opcode(raw, decoded.mnemonic), Some(expected));
    }
}
