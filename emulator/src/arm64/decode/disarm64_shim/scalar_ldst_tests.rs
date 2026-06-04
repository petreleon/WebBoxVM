use super::*;

#[test]
fn maps_scalar_byte_halfword_load_store_mnemonics() {
    let cases = [
        (0x3940_0C41, Opcode::Ldr, "ldrb"),
        (0x7940_0C83, Opcode::Ldr, "ldrh"),
        (0x785F_E18B, Opcode::Ldr, "ldurh"),
        (0x3980_1CC5, Opcode::LdrSign, "ldrsb"),
        (0x79C0_1107, Opcode::LdrSign, "ldrsh"),
        (0x38DF_F149, Opcode::LdrSign, "ldursb"),
        (0x789F_A041, Opcode::LdrSign, "ldursh"),
        (0xB89F_8083, Opcode::LdrSign, "ldursw"),
        (0x3900_25CD, Opcode::Str, "strb"),
        (0x7900_160F, Opcode::Str, "strh"),
        (0x381F_D251, Opcode::Str, "sturb"),
        (0x781F_C293, Opcode::Str, "sturh"),
    ];

    for (raw, expected, mnemonic) in cases {
        let decoded = decoder::decode(raw).expect("disarm64 should decode scalar load/store");
        assert_eq!(format!("{:?}", decoded.mnemonic), mnemonic);
        assert_eq!(mnemonic_to_opcode(raw, decoded.mnemonic), Some(expected));
    }
}
