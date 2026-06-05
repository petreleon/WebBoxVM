use super::*;

#[test]
fn decode_br_x0() {
    let instr = decode(0xD61F0000).unwrap();
    assert_eq!(instr.op, Opcode::Br);
    assert_eq!(instr.rn, 0);
}

#[test]
fn decode_ret() {
    let instr = decode(0xD65F03C0).unwrap();
    assert_eq!(instr.op, Opcode::Ret);
    assert_eq!(instr.rn, 30);
}

#[test]
fn decode_blr() {
    let instr = decode(0xD63F0000).unwrap();
    assert_eq!(instr.op, Opcode::Blr);
    assert_eq!(instr.rn, 0);
}

#[test]
fn decode_pointer_authenticated_branches_as_register_branches() {
    let cases = [
        (0xD71F_0821, Opcode::Br, "braa", 1),
        (0xD71F_0C22, Opcode::Br, "brab", 1),
        (0xD73F_0843, Opcode::Blr, "blraa", 2),
        (0xD73F_0C44, Opcode::Blr, "blrab", 2),
        (0xD61F_081F, Opcode::Br, "braaz", 0),
        (0xD61F_0C1F, Opcode::Br, "brabz", 0),
        (0xD63F_081F, Opcode::Blr, "blraaz", 0),
        (0xD63F_0C1F, Opcode::Blr, "blrabz", 0),
    ];

    for (raw, expected, mnemonic, rn) in cases {
        assert_disarm64_mnemonic(raw, mnemonic);
        let instr = decode(raw).unwrap();
        assert_eq!(instr.op, expected, "raw=0x{raw:08x}");
        assert_eq!(instr.rn, rn, "raw=0x{raw:08x}");
    }
}

#[test]
fn decode_authenticated_exception_returns_as_eret() {
    for (raw, mnemonic) in [(0xD69F_0BFF, "eretaa"), (0xD69F_0FFF, "eretab")] {
        assert_disarm64_mnemonic(raw, mnemonic);
        assert_eq!(decode(raw).unwrap().op, Opcode::Eret, "raw=0x{raw:08x}");
    }
}

#[test]
fn decode_conditional_branch_cross_checked_with_disarm64() {
    let cases = [
        (0x5400_0000, 0, 0),
        (0x5400_0001, 1, 0),
        (0x5400_002A, 10, 4),
    ];

    for (raw, cond, imm) in cases {
        assert_disarm64_mnemonic(raw, "b_");
        let instr = decode(raw).unwrap();
        assert_eq!(instr.op, Opcode::BCond, "raw=0x{raw:08x}");
        assert_eq!(instr.cond, cond, "raw=0x{raw:08x}");
        assert_eq!(instr.imm, imm, "raw=0x{raw:08x}");
    }
}

#[test]
fn decode_ccmp_imm_pl_imm_d() {
    let raw: u32 = 0xFA405A4D;
    let instr = decode(raw).unwrap();
    assert_eq!(instr.op, Opcode::Ccmp);
    assert_eq!(instr.cond, 5); // PL
    assert_eq!(instr.imm, 0xD); // nzcv
    assert_eq!(instr.size, 1); // immediate operand
}

#[test]
fn decode_ccmn_immediate() {
    let instr = decode(0x3A48_0960).unwrap(); // ccmn w11, #8, #0, eq
    assert_eq!(instr.op, Opcode::Ccmn);
    assert_eq!(instr.rn, 11);
    assert_eq!(instr.rm, 8);
    assert_eq!(instr.cond, 0);
    assert_eq!(instr.size, 1);
}
