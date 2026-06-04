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
        (0x0E22_3420, Opcode::SimdCmgtReg, "cmgt"),
        (0x0E22_3C20, Opcode::SimdCmgeReg, "cmge"),
        (0x5EE0_3420, Opcode::SimdCmgtReg, "cmgt"),
        (0x5EE0_3C20, Opcode::SimdCmgeReg, "cmge"),
        (0x5EE0_8C20, Opcode::SimdCmtst, "cmtst"),
        (0x7EE0_3C20, Opcode::SimdCmhsReg, "cmhs"),
        (0x7EE0_8C20, Opcode::SimdCmeqReg, "cmeq"),
        (0x4EA0_8821, Opcode::SimdCmgtZero, "cmgt"),
        (0x6EA0_9821, Opcode::SimdCmleZero, "cmle"),
        (0x4EA0_A821, Opcode::SimdCmltZero, "cmlt"),
        (0x0E20_6C20, Opcode::SimdSminVec, "smin"),
        (0x0E20_A420, Opcode::SimdSmaxp, "smaxp"),
        (0x0E20_AC20, Opcode::SimdSminp, "sminp"),
        (0x2E20_A420, Opcode::SimdUmaxp, "umaxp"),
        (0x0E30_A820, Opcode::SimdSmaxv, "smaxv"),
        (0x0E31_A820, Opcode::SimdSminv, "sminv"),
        (0x2E31_A820, Opcode::SimdUminv, "uminv"),
        (0x6E30_F820, Opcode::SimdFpFmaxv, "fmaxv"),
        (0x6EB0_F862, Opcode::SimdFpFminv, "fminv"),
        (0x6E30_C8A4, Opcode::SimdFpFmaxnmv, "fmaxnmv"),
        (0x6EB0_C8E6, Opcode::SimdFpFminnmv, "fminnmv"),
        (0x4E9C_5BDE, Opcode::SimdUzp2, "uzp2"),
        (0x4E0C_690B, Opcode::SimdTrn2, "trn2"),
        (0x4EA1_2BEF, Opcode::SimdXtn2, "xtn2"),
    ];

    for (raw, expected, mnemonic) in cases {
        let decoded = decoder::decode(raw).expect("disarm64 should decode SIMD mnemonic word");
        assert_eq!(format!("{:?}", decoded.mnemonic), mnemonic);
        assert_eq!(mnemonic_to_opcode(raw, decoded.mnemonic), Some(expected));
    }

    for raw in [0x2EE0_A420, 0x2EE0_AC20, 0x0EB0_A820, 0x6EF1_A820] {
        let decoded = decoder::decode(raw).expect("disarm64 should flag undefined SIMD form");
        assert_eq!(mnemonic_to_opcode(raw, decoded.mnemonic), None);
    }
}
