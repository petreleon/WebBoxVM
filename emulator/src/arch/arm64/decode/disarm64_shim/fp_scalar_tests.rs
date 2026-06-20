use super::*;

#[test]
fn maps_scalar_fp_minmax_mnemonics() {
    let cases = [
        (0x1E22_4820, Opcode::FpMax, "fmax"),
        (0x1E25_5883, Opcode::FpMin, "fmin"),
        (0x1E68_48E6, Opcode::FpMax, "fmax"),
        (0x1E6B_5949, Opcode::FpMin, "fmin"),
    ];

    for (raw, expected, mnemonic) in cases {
        let decoded = decoder::decode(raw).expect("disarm64 should decode scalar FP min/max");
        assert_eq!(format!("{:?}", decoded.mnemonic), mnemonic);
        assert_eq!(mnemonic_to_opcode(raw, decoded.mnemonic), Some(expected));
    }
}

#[test]
fn leaves_unsupported_half_fixed_fcvtzu_unmapped() {
    let raw = 0x7F5B_FFB3;
    let decoded = decoder::decode(raw).expect("disarm64 should decode half fcvtzu");
    assert_eq!(format!("{:?}", decoded.mnemonic), "fcvtzu");
    assert_eq!(mnemonic_to_opcode(raw, decoded.mnemonic), None);
}
