use super::*;

#[test]
fn maps_sve_fp_convert_mnemonics_by_encoding() {
    let cases = [
        (0x6594_AFFF, Opcode::SveScvtf),
        (0x65D0_A020, Opcode::SveScvtf),
        (0x65D4_A020, Opcode::SveScvtf),
        (0x65D6_A3DE, Opcode::SveScvtf),
        (0x659C_A3FF, Opcode::SveFcvtzs),
        (0x65DC_A020, Opcode::SveFcvtzs),
        (0x65D8_A020, Opcode::SveFcvtzs),
        (0x65DE_A39A, Opcode::SveFcvtzs),
    ];

    for (raw, expected) in cases {
        let decoded = decoder::decode(raw).expect("disarm64 should decode SVE FP convert word");
        assert_eq!(mnemonic_to_opcode(raw, decoded.mnemonic), Some(expected));
    }
}
