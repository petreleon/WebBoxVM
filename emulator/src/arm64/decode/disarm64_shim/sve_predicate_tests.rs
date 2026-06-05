use super::*;

#[test]
fn maps_sve_predicate_mnemonics_by_encoding() {
    let cases = [
        (0x2522_0CE1, Opcode::SveWhileLo),
        (0x2522_1CE1, Opcode::SveWhileLo),
        (0x2522_04A1, Opcode::SveWhileLt),
        (0x2522_04B1, Opcode::SveWhileLe),
        (0x2522_0CB1, Opcode::SveWhileLs),
        (0x2562_0FE0, Opcode::SveWhileLo),
        (0x25A6_0FE1, Opcode::SveWhileLo),
        (0x25E9_1CA3, Opcode::SveWhileLo),
        (0x2599_E3E0, Opcode::SvePtrues),
    ];

    for (raw, expected) in cases {
        let decoded = decoder::decode(raw).expect("disarm64 should decode SVE predicate word");
        assert_eq!(mnemonic_to_opcode(raw, decoded.mnemonic), Some(expected));
    }
}
