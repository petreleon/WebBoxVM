use super::*;

#[test]
fn maps_mte_mnemonics_by_encoding() {
    let cases = [
        (0xD960_0000, Opcode::MteLdg, "ldg"),
        (0x9AC1_1000, Opcode::MteIrg, "irg"),
        (0x9ADF_1401, Opcode::MteGmi, "gmi"),
        (0xD920_0800, Opcode::MteStg, "stg"),
        (0xD960_0800, Opcode::MteStzg, "stzg"),
        (0xD9A0_0800, Opcode::MteSt2g, "st2g"),
        (0xD9E0_0800, Opcode::MteStz2g, "stz2g"),
    ];

    for (raw, expected, mnemonic) in cases {
        let decoded = decoder::decode(raw).expect("disarm64 should decode MTE word");
        assert_eq!(format!("{:?}", decoded.mnemonic), mnemonic);
        assert_eq!(mnemonic_to_opcode(raw, decoded.mnemonic), Some(expected));
    }
}
