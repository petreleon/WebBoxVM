use super::*;

#[test]
fn maps_sve_fp_unary_mnemonics_by_encoding() {
    let cases = [
        (0x6580_A3DE, Opcode::SveFpFrintn),
        (0x65C4_A39C, Opcode::SveFpFrinta),
        (0x6583_BFFE, Opcode::SveFpFrintz),
        (0x65CD_ABFB, Opcode::SveFpSqrt),
    ];

    for (raw, expected) in cases {
        let decoded = decoder::decode(raw).expect("disarm64 should decode SVE FP unary word");
        assert_eq!(mnemonic_to_opcode(raw, decoded.mnemonic), Some(expected));
    }
}
