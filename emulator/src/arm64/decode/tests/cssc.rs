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

#[test]
fn decode_cssc_scalar_minmax_register_forms() {
    let cases = [
        (0x1AC2_6020, Opcode::Smax, false, "smax"),
        (0x9AC2_6020, Opcode::Smax, true, "smax"),
        (0x1AC2_6820, Opcode::Smin, false, "smin"),
        (0x9AC2_6820, Opcode::Smin, true, "smin"),
        (0x1AC2_6420, Opcode::Umax, false, "umax"),
        (0x9AC2_6420, Opcode::Umax, true, "umax"),
        (0x1AC2_6C20, Opcode::Umin, false, "umin"),
        (0x9AC2_6C20, Opcode::Umin, true, "umin"),
    ];

    for (raw, expected, sf, mnemonic) in cases {
        assert_disarm64_mnemonic(raw, mnemonic);
        let instr = decode(raw).unwrap();
        assert_eq!(instr.op, expected);
        assert_eq!((instr.rd, instr.rn, instr.rm, instr.sf), (0, 1, 2, sf));
    }
}

#[test]
fn decode_cssc_scalar_minmax_immediate_forms() {
    let cases = [
        (0x11C3_FC20, Opcode::Smax, false, (-1i64) as u64, "smax"),
        (0x91C0_0020, Opcode::Smax, true, 0, "smax"),
        (0x11CA_0020, Opcode::Smin, false, (-128i64) as u64, "smin"),
        (0x91C9_FC20, Opcode::Smin, true, 127, "smin"),
        (0x11C7_FC20, Opcode::Umax, false, 255, "umax"),
        (0x91C4_0020, Opcode::Umax, true, 0, "umax"),
        (0x11CF_FC20, Opcode::Umin, false, 255, "umin"),
        (0x91CC_0020, Opcode::Umin, true, 0, "umin"),
    ];

    for (raw, expected, sf, imm, mnemonic) in cases {
        assert_disarm64_mnemonic(raw, mnemonic);
        let instr = decode(raw).unwrap();
        assert_eq!(instr.op, expected);
        assert_eq!((instr.rd, instr.rn, instr.rm, instr.sf), (0, 1, 0xFF, sf));
        assert_eq!(instr.imm, imm);
    }
}
