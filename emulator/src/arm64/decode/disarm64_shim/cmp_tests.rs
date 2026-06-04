use super::*;

#[test]
fn maps_shifted_register_cmp_aliases() {
    let cases = [
        (0xEB00_003F, Opcode::Cmp),
        (0x6B00_00BF, Opcode::Cmp),
        (0xEB41_47FF, Opcode::Cmp),
        (0xEB02_007E, Opcode::Subs),
    ];

    for (raw, expected) in cases {
        let decoded = decoder::decode(raw).expect("disarm64 should decode subs word");
        assert_eq!(format!("{:?}", decoded.mnemonic), "subs");
        assert_eq!(mnemonic_to_opcode(raw, decoded.mnemonic), Some(expected));
    }
}
