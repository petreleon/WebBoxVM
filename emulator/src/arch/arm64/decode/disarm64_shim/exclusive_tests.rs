use super::*;

#[test]
fn maps_exclusive_mnemonics() {
    let cases = [
        (0xC85F_7C40, Opcode::Ldxr, "ldxr"),
        (0xC85F_FC40, Opcode::Ldxr, "ldaxr"),
        (0x485F_FC83, Opcode::Ldxr, "ldaxrh"),
        (0x085F_FCC5, Opcode::Ldxr, "ldaxrb"),
        (0xC81F_7C40, Opcode::Stxr, "stxr"),
        (0xC807_FD28, Opcode::Stxr, "stlxr"),
        (0x480A_FD8B, Opcode::Stxr, "stlxrh"),
        (0x080D_FDEE, Opcode::Stxr, "stlxrb"),
        (0xC8DF_FC40, Opcode::Ldar, "ldar"),
        (0x48DF_FE30, Opcode::Ldar, "ldarh"),
        (0x99C0_0820, Opcode::Ldar, "ldapr"),
        (0xD9C0_0862, Opcode::Ldar, "ldapr"),
        (0x089F_FE72, Opcode::Stlr, "stlrb"),
        (0x9980_08A4, Opcode::Stlr, "stlr"),
        (0xD980_08E6, Opcode::Stlr, "stlr"),
        (0xC87F_8440, Opcode::Ldxp, "ldaxp"),
        (0xC823_8440, Opcode::Stxp, "stlxp"),
        (0xC82E_7FAE, Opcode::Stxp, "stxp"),
    ];

    for (raw, expected, mnemonic) in cases {
        let decoded = decoder::decode(raw).expect("disarm64 should decode exclusive word");
        assert_eq!(format!("{:?}", decoded.mnemonic), mnemonic);
        assert_eq!(mnemonic_to_opcode(raw, decoded.mnemonic), Some(expected));
    }
}
