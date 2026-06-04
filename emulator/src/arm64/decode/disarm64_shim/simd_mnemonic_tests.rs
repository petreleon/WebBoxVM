use super::*;

#[test]
fn maps_shared_simd_mnemonics_by_encoding() {
    let cases = [
        (0x6E3E_1FFE, Opcode::SimdEor, "eor"),
        (0x6E3F_1FBD, Opcode::SimdEor, "eor"),
        (0x2E20_1C00, Opcode::SimdEor, "eor"),
        (0x5200_9CA4, Opcode::EorImm, "eor"),
        (0xCA28_00E6, Opcode::EorReg, "eon"),
        (0x6F00_041F, Opcode::SimdMovi, "mvni"),
        (0x2F03_D7FE, Opcode::SimdMvni, "mvni"),
        (0x4E9C_5BDE, Opcode::SimdUzp2, "uzp2"),
        (0x4E0C_690B, Opcode::SimdTrn2, "trn2"),
        (0x4EA1_2BEF, Opcode::SimdXtn2, "xtn2"),
    ];

    for (raw, expected, mnemonic) in cases {
        let decoded = decoder::decode(raw).expect("disarm64 should decode SIMD mnemonic word");
        assert_eq!(format!("{:?}", decoded.mnemonic), mnemonic);
        assert_eq!(mnemonic_to_opcode(raw, decoded.mnemonic), Some(expected));
    }
}
