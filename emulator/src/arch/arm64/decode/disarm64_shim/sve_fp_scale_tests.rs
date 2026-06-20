use super::*;

#[test]
fn maps_sve_fscale_mnemonics_by_encoding() {
    for raw in [0x6549_8FE1, 0x6589_8FE1, 0x65C9_8FE1] {
        let decoded = decoder::decode(raw).expect("disarm64 should decode FSCALE word");
        assert_eq!(
            mnemonic_to_opcode(raw, decoded.mnemonic),
            Some(Opcode::SveFpFscale)
        );
    }
}
