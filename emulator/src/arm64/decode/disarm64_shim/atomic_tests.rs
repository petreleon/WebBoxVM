use super::*;

#[test]
fn maps_atomic_mnemonics() {
    let cases = [
        (0x4860_FC82, Opcode::Casp, "caspal"),
        (0x88E1_7C02, Opcode::Cas, "casa"),
        (0xB8E1_0001, Opcode::Atomic, "ldaddal"),
        (0xF8A0_3260, Opcode::Atomic, "ldseta"),
        (0x1921_80C0, Opcode::AtomicPair, "swpp"),
        (0x19A3_3002, Opcode::AtomicPair, "ldsetpa"),
        (0x19E7_10A6, Opcode::AtomicPair, "ldclrpal"),
    ];

    for (raw, expected, mnemonic) in cases {
        let decoded = decoder::decode(raw).expect("disarm64 should decode atomic word");
        assert_eq!(format!("{:?}", decoded.mnemonic), mnemonic);
        assert_eq!(mnemonic_to_opcode(raw, decoded.mnemonic), Some(expected));
    }
}

#[test]
fn leaves_constrained_pair_atomics_unmapped() {
    for raw in [0x1921_80DF, 0x193F_80C0, 0x1920_80C0] {
        let decoded = decoder::decode(raw).expect("disarm64 should decode pair atomic word");
        assert_eq!(format!("{:?}", decoded.mnemonic), "swpp");
        assert_eq!(mnemonic_to_opcode(raw, decoded.mnemonic), None);
    }
}
