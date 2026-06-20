use super::*;

#[test]
fn maps_scalar_cssc_mnemonics() {
    let cases = [
        (0x5AC0_2020, Opcode::Abs, "abs"),
        (0xDAC0_2020, Opcode::Abs, "abs"),
        (0x5AC0_1820, Opcode::Ctz, "ctz"),
        (0xDAC0_1820, Opcode::Ctz, "ctz"),
        (0x5AC0_1C20, Opcode::Cnt, "cnt"),
        (0xDAC0_1C20, Opcode::Cnt, "cnt"),
        (0x1AC2_6020, Opcode::Smax, "smax"),
        (0x9AC2_6820, Opcode::Smin, "smin"),
        (0x1AC2_6420, Opcode::Umax, "umax"),
        (0x9AC2_6C20, Opcode::Umin, "umin"),
        (0x11C3_FC20, Opcode::Smax, "smax"),
        (0x91C8_0020, Opcode::Smin, "smin"),
        (0x11C7_FC20, Opcode::Umax, "umax"),
        (0x91CC_0020, Opcode::Umin, "umin"),
    ];

    for (raw, expected, mnemonic) in cases {
        let decoded = decoder::decode(raw).expect("disarm64 should decode CSSC word");
        assert_eq!(format!("{:?}", decoded.mnemonic), mnemonic);
        assert_eq!(mnemonic_to_opcode(raw, decoded.mnemonic), Some(expected));
    }
}
