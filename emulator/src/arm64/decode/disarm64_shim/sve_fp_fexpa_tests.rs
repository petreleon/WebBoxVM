use super::*;

#[test]
fn maps_sve_fexpa_mnemonics_by_encoding() {
    let cases = [
        0x0460_B800,
        0x0460_B821,
        0x04A0_B800,
        0x04E0_B800,
        0x04E0_BBFF,
    ];

    for raw in cases {
        let decoded = decoder::decode(raw).expect("disarm64 should decode FEXPA word");
        assert_eq!(
            mnemonic_to_opcode(raw, decoded.mnemonic),
            Some(Opcode::SveFpFexpa)
        );
    }
}
