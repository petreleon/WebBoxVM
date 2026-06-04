use super::*;

#[test]
fn decode_ldrsw_unsigned_offset() {
    let instr = decode(0xB980_27F9).unwrap(); // ldrsw x25, [sp, #36]
    assert_eq!(instr.op, Opcode::LdrSign);
    assert_eq!(instr.rd, 25);
    assert_eq!(instr.rn, 31);
    assert_eq!(instr.imm, 36);
    assert!(instr.sf);
}

#[test]
fn decode_literal_loads_cross_checked_with_disarm64() {
    let cases = [
        (0x1800_0003, "ldr", 3, 0, false, 0),
        (0x5800_0004, "ldr", 4, 0, true, 0),
        (0x9800_0005, "ldrsw", 5, 0, true, 1),
        (0x1C00_0006, "ldr", 6, 4, true, 0),
        (0x5C00_0007, "ldr", 7, 8, true, 0),
        (0x9C00_0008, "ldr", 8, 16, true, 0),
    ];

    for (raw, mnemonic, rd, size, sf, cond) in cases {
        assert_disarm64_mnemonic(raw, mnemonic);
        let instr = decode(raw).unwrap();
        assert_eq!(instr.op, Opcode::LdrLit, "raw=0x{raw:08x}");
        assert_eq!(instr.rd, rd, "raw=0x{raw:08x}");
        assert_eq!(instr.size, size, "raw=0x{raw:08x}");
        assert_eq!(instr.sf, sf, "raw=0x{raw:08x}");
        assert_eq!(instr.cond, cond, "raw=0x{raw:08x}");
        assert_eq!(instr.imm, 0, "raw=0x{raw:08x}");
    }

    let q_forward = decode(0x9C00_1DA8).unwrap();
    assert_eq!(q_forward.op, Opcode::LdrLit);
    assert_eq!(q_forward.rd, 8);
    assert_eq!(q_forward.size, 16);
    assert_eq!(q_forward.imm, 0x3b4);

    let q_backward = decode(0x9CFF_FA08).unwrap();
    assert_eq!(q_backward.op, Opcode::LdrLit);
    assert_eq!(q_backward.rd, 8);
    assert_eq!(q_backward.size, 16);
    assert_eq!(q_backward.imm as i64, -0xc0);

    assert!(decode(0x5EA0_D400).is_none());
    assert!(decode(0xDC00_0000).is_none());
    assert_eq!(decode(0xD800_0000).unwrap().op, Opcode::Nop);
}
