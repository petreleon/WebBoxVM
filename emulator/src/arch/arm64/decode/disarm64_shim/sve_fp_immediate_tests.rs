use super::*;

#[test]
fn maps_sve_fp_immediate_mnemonics_by_encoding() {
    let cases = [
        (0x05D3_CE1F, Opcode::SveFpCpyImm),
        (0x0592_C01D, Opcode::SveFpCpyImm),
        (0x0592_CE1A, Opcode::SveFpCpyImm),
    ];

    for (raw, expected) in cases {
        let decoded = decoder::decode(raw).expect("disarm64 should decode SVE FCPY word");
        assert_eq!(mnemonic_to_opcode(raw, decoded.mnemonic), Some(expected));
    }
}
