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
        (0x4E22_F420, Opcode::SimdFpFmaxVec, "fmax"),
        (0x4EE5_F483, Opcode::SimdFpFminVec, "fmin"),
        (0x4E28_C4E6, Opcode::SimdFpFmaxnmVec, "fmaxnm"),
        (0x4EEB_C549, Opcode::SimdFpFminnmVec, "fminnm"),
        (0x6E22_F420, Opcode::SimdFpFmaxp, "fmaxp"),
        (0x6EE5_F483, Opcode::SimdFpFminp, "fminp"),
        (0x6E28_C4E6, Opcode::SimdFpFmaxnmp, "fmaxnmp"),
        (0x6EEB_C549, Opcode::SimdFpFminnmp, "fminnmp"),
        (0x7E30_F820, Opcode::SimdFpFmaxp, "fmaxp"),
        (0x7EB0_F862, Opcode::SimdFpFminp, "fminp"),
        (0x7E30_C8A4, Opcode::SimdFpFmaxnmp, "fmaxnmp"),
        (0x7EB0_C8E6, Opcode::SimdFpFminnmp, "fminnmp"),
        (0x7E70_F928, Opcode::SimdFpFmaxp, "fmaxp"),
        (0x7EF0_F96A, Opcode::SimdFpFminp, "fminp"),
        (0x7E70_C9AC, Opcode::SimdFpFmaxnmp, "fmaxnmp"),
        (0x7EF0_C9EE, Opcode::SimdFpFminnmp, "fminnmp"),
        (0x6E25_D483, Opcode::SimdFpAddp, "faddp"),
        (0x7E30_D949, Opcode::SimdFpAddp, "faddp"),
        (0x7E70_D98B, Opcode::SimdFpAddp, "faddp"),
        (0x0E7E_13BD, Opcode::SimdSaddw, "saddw"),
        (0x4E7E_13FF, Opcode::SimdSaddw, "saddw2"),
        (0x2EBF_13DF, Opcode::SimdUaddw, "uaddw"),
        (0x6EA8_10E6, Opcode::SimdUaddw, "uaddw2"),
        (0x5E22_DC20, Opcode::SimdFpMulx, "fmulx"),
        (0x4E2B_DD49, Opcode::SimdFpMulx, "fmulx"),
        (0x4E6E_DDAC, Opcode::SimdFpMulx, "fmulx"),
        (0x2F82_9020, Opcode::SimdFpMulxElem, "fmulx"),
        (0x7F8B_9949, Opcode::SimdFpMulxElem, "fmulx"),
        (0x0E22_E420, Opcode::SimdFpFcmeqVec, "fcmeq"),
        (0x4E25_E483, Opcode::SimdFpFcmeqVec, "fcmeq"),
        (0x4E68_E4E6, Opcode::SimdFpFcmeqVec, "fcmeq"),
        (0x5E2B_E549, Opcode::SimdFpFcmeqVec, "fcmeq"),
        (0x5E6E_E5AC, Opcode::SimdFpFcmeqVec, "fcmeq"),
        (0x7E22_E420, Opcode::SimdFpFcmgeVec, "fcmge"),
        (0x7EA5_E483, Opcode::SimdFpFcmgtVec, "fcmgt"),
        (0x7E28_ECE6, Opcode::SimdFpFacgeVec, "facge"),
        (0x7EAB_ED49, Opcode::SimdFpFacgtVec, "facgt"),
        (0x2EA0_C820, Opcode::SimdFpFcmgeZero, "fcmge"),
        (0x4EA0_C862, Opcode::SimdFpFcmgtZero, "fcmgt"),
        (0x7EA0_C820, Opcode::SimdFpFcmgeZero, "fcmge"),
        (0x5EA0_C862, Opcode::SimdFpFcmgtZero, "fcmgt"),
        (0x5EA0_D9AC, Opcode::SimdFpFcmeqZero, "fcmeq"),
        (0x7EA0_D9EE, Opcode::SimdFpFcmleZero, "fcmle"),
        (0x5EA0_EA30, Opcode::SimdFpFcmltZero, "fcmlt"),
        (0x0E20_4820, Opcode::SimdCls, "cls"),
        (0x6EA0_4820, Opcode::SimdClz, "clz"),
        (0x4E9C_5BDE, Opcode::SimdUzp2, "uzp2"),
        (0x4E0C_690B, Opcode::SimdTrn2, "trn2"),
        (0x4EA1_2BEF, Opcode::SimdXtn2, "xtn2"),
    ];

    for (raw, expected, mnemonic) in cases {
        let decoded = decoder::decode(raw).expect("disarm64 should decode SIMD mnemonic word");
        assert_eq!(format!("{:?}", decoded.mnemonic), mnemonic);
        assert_eq!(mnemonic_to_opcode(raw, decoded.mnemonic), Some(expected));
    }

    for raw in [
        0x2EE0_A420,
        0x2EE0_AC20,
        0x0EB0_A820,
        0x6EF1_A820,
        0x0E62_F420,
        0x0E62_C420,
    ] {
        let decoded = decoder::decode(raw).expect("disarm64 should flag undefined SIMD form");
        assert_eq!(mnemonic_to_opcode(raw, decoded.mnemonic), None);
    }
}

#[test]
fn maps_simd_rcpc_unscaled_mnemonics_by_encoding() {
    let cases = [
        (0x1D40_0820, Opcode::SimdLdr, "ldapur"),
        (0x1DC0_4884, Opcode::SimdLdr, "ldapur"),
        (0x1D00_0820, Opcode::SimdStr, "stlur"),
        (0x1D80_4884, Opcode::SimdStr, "stlur"),
        (0x0D41_8420, Opcode::SimdLd1Lane, "ldap1"),
        (0x4D41_8420, Opcode::SimdLd1Lane, "ldap1"),
        (0x0D01_8462, Opcode::SimdSt1Lane, "stl1"),
        (0x4D01_8462, Opcode::SimdSt1Lane, "stl1"),
    ];

    for (raw, expected, mnemonic) in cases {
        let decoded = decoder::decode(raw).expect("disarm64 should decode RCpc SIMD memory");
        assert_eq!(format!("{:?}", decoded.mnemonic), mnemonic);
        assert_eq!(mnemonic_to_opcode(raw, decoded.mnemonic), Some(expected));
    }
}
