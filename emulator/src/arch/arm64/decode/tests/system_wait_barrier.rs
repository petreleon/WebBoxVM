use super::*;

#[test]
fn decode_wait_timeout_and_hint_barriers() {
    let cases = [
        (0xD503_20DF, Opcode::Dgh, "dgh", 0),
        (0xD503_30FF, Opcode::Sb, "sb", 0),
        (0xD503_1005, Opcode::Wfe, "wfet", 5),
        (0xD503_1025, Opcode::Wfi, "wfit", 5),
    ];

    for (raw, expected, mnemonic, rd) in cases {
        assert_disarm64_mnemonic(raw, mnemonic);
        let instr = decode(raw).unwrap();
        assert_eq!(instr.op, expected, "raw=0x{raw:08x}");
        assert_eq!(instr.rd, rd, "raw=0x{raw:08x}");
    }
}

#[test]
fn decode_yield_hint_as_named_noop() {
    let raw = 0xD503_203F;
    let decoded = disarm64::decoder::decode(raw).unwrap();
    assert_eq!(decoded.to_string(), "yield");
    assert_eq!(decode(raw).unwrap().op, Opcode::Yield);
}
