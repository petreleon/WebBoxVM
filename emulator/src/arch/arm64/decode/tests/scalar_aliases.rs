use super::*;

#[test]
fn decode_mov_register_aliases_cross_checked_with_disarm64() {
    for (raw, rd, rm, sf) in [
        (0xAA01_03E0, 0, 1, true),
        (0x2A01_03E0, 0, 1, false),
        (0xAA1F_03E0, 0, 31, true),
    ] {
        assert_disarm64_mnemonic(raw, "orr");
        let instr = decode(raw).unwrap();
        assert_eq!(instr.op, Opcode::MovReg, "raw=0x{raw:08x}");
        assert_eq!(instr.rd, rd, "raw=0x{raw:08x}");
        assert_eq!(instr.rm, rm, "raw=0x{raw:08x}");
        assert_eq!(instr.sf, sf, "raw=0x{raw:08x}");
    }
}

#[test]
fn decode_extract_and_sxtw_aliases_cross_checked_with_disarm64() {
    let sxtw = decode(0x9340_7C62).unwrap();
    assert_disarm64_mnemonic(0x9340_7C62, "sbfm");
    assert_eq!(sxtw.op, Opcode::Sxtw);
    assert_eq!(sxtw.rd, 2);
    assert_eq!(sxtw.rn, 3);
    assert_eq!(sxtw.imm, 32);

    for raw in [0x1381_0820, 0x1384_1C62] {
        assert_disarm64_mnemonic(raw, "extr");
        assert_eq!(decode(raw).unwrap().op, Opcode::Extr, "raw=0x{raw:08x}");
    }
}
