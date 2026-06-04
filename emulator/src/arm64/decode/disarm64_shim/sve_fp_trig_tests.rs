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

#[test]
fn maps_sve_ftsmul_and_ftssel_mnemonics_by_encoding() {
    let cases = [
        (0x6543_0C41, Opcode::SveFpFtsmul),
        (0x6583_0C41, Opcode::SveFpFtsmul),
        (0x65C3_0C41, Opcode::SveFpFtsmul),
        (0x0463_B041, Opcode::SveFpFtssel),
        (0x04A3_B041, Opcode::SveFpFtssel),
        (0x04E3_B041, Opcode::SveFpFtssel),
    ];

    for (raw, op) in cases {
        let decoded = decoder::decode(raw).expect("disarm64 should decode trig word");
        assert_eq!(mnemonic_to_opcode(raw, decoded.mnemonic), Some(op));
    }
}
