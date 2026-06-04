use super::*;

#[test]
fn decode_cssc_scalar_unary_forms() {
    let cases = [
        (0x5AC0_2020, Opcode::Abs, false, "abs"),
        (0xDAC0_2020, Opcode::Abs, true, "abs"),
        (0x5AC0_1820, Opcode::Ctz, false, "ctz"),
        (0xDAC0_1820, Opcode::Ctz, true, "ctz"),
        (0x5AC0_1C20, Opcode::Cnt, false, "cnt"),
        (0xDAC0_1C20, Opcode::Cnt, true, "cnt"),
    ];

    for (raw, expected, sf, mnemonic) in cases {
        assert_disarm64_mnemonic(raw, mnemonic);
        let instr = decode(raw).unwrap();
        assert_eq!(instr.op, expected);
        assert_eq!((instr.rd, instr.rn, instr.sf), (0, 1, sf));
    }
}
