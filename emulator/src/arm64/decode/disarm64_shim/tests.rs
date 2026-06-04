use super::*;

#[test]
fn maps_simd_pairwise_narrow_mnemonics() {
    let cases = [
        (0x6E22_3C20, Opcode::SimdCmhsReg, "cmhs"),
        (0x0E28_40E6, Opcode::SimdAddhn, "addhn"),
        (0x6E25_A483, Opcode::SimdUmaxp, "umaxp"),
    ];

    for (raw, expected, mnemonic) in cases {
        let decoded = decoder::decode(raw).expect("disarm64 should decode test word");
        assert_eq!(format!("{:?}", decoded.mnemonic), mnemonic);
        assert_eq!(mnemonic_to_opcode(raw, decoded.mnemonic), Some(expected));
    }
}

#[test]
fn leaves_addhn2_unmapped_until_upper_half_execution_exists() {
    let raw = 0x4E2B_4149;
    let decoded = decoder::decode(raw).expect("disarm64 should decode addhn2");

    assert_eq!(format!("{:?}", decoded.mnemonic), "addhn2");
    assert_eq!(mnemonic_to_opcode(raw, decoded.mnemonic), None);
}

#[test]
fn maps_scalar_variable_rotate_mnemonic() {
    let raw = 0x9AC2_2C20;
    let decoded = decoder::decode(raw).expect("disarm64 should decode rorv");

    assert_eq!(format!("{:?}", decoded.mnemonic), "rorv");
    assert_eq!(
        mnemonic_to_opcode(raw, decoded.mnemonic),
        Some(Opcode::Rorv)
    );
}

#[test]
fn maps_scalar_reverse_mnemonics() {
    let cases = [
        (0x5AC0_04E6, Opcode::Rev16, "rev16"),
        (0xDAC0_04A4, Opcode::Rev16, "rev16"),
        (0xDAC0_0928, Opcode::Rev32, "rev32"),
    ];

    for (raw, expected, mnemonic) in cases {
        let decoded = decoder::decode(raw).expect("disarm64 should decode reverse word");
        assert_eq!(format!("{:?}", decoded.mnemonic), mnemonic);
        assert_eq!(mnemonic_to_opcode(raw, decoded.mnemonic), Some(expected));
    }
}

#[test]
fn maps_scalar_bitfield_mnemonics() {
    let cases = [
        (0x9343_3020, Opcode::Sbfm, "sbfm"),
        (0xB348_3CA4, Opcode::Bfm, "bfm"),
        (0xD344_5062, Opcode::Ubfm, "ubfm"),
    ];

    for (raw, expected, mnemonic) in cases {
        let decoded = decoder::decode(raw).expect("disarm64 should decode bitfield word");
        assert_eq!(format!("{:?}", decoded.mnemonic), mnemonic);
        assert_eq!(mnemonic_to_opcode(raw, decoded.mnemonic), Some(expected));
    }
}
