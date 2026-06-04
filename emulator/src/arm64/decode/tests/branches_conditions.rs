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
