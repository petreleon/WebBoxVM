use super::*;

#[test]
fn maps_sve_ftmad_mnemonics_by_encoding() {
    for raw in [0x6550_83C1, 0x6594_83C1, 0x65D7_83C1] {
        let decoded = decoder::decode(raw).expect("disarm64 should decode FTMAD word");
        assert_eq!(
            mnemonic_to_opcode(raw, decoded.mnemonic),
            Some(Opcode::SveFpFtmad)
        );
    }
}
