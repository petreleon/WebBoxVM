use super::*;

#[test]
fn maps_sve_xar_mnemonic_by_encoding() {
    for raw in [0x042D_34C5, 0x0433_3421, 0x0470_3403, 0x04E0_3462] {
        let decoded = decoder::decode(raw).expect("disarm64 should decode SVE XAR word");
        assert_eq!(
            mnemonic_to_opcode(raw, decoded.mnemonic),
            Some(Opcode::SveXar)
        );
    }
}
