use super::*;

#[test]
fn maps_simd_ucvtf_by_encoding() {
    let cases = [
        (0x7E61_DBDE, "ucvtf"),
        (0x7E21_D821, "ucvtf"),
        (0x6E61_D800, "ucvtf"),
        (0x6E21_D821, "ucvtf"),
        (0x2E21_D842, "ucvtf"),
    ];

    for (raw, mnemonic) in cases {
        let decoded = decoder::decode(raw).expect("disarm64 should decode ucvtf word");
        assert_eq!(format!("{:?}", decoded.mnemonic), mnemonic);
        assert_eq!(
            mnemonic_to_opcode(raw, decoded.mnemonic),
            Some(Opcode::SimdUcvtf)
        );
    }
}
