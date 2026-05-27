use super::*;

#[test]
fn movz_lsl_0() {
    let instr = decode(0xD282_4680).unwrap();
    assert_eq!(instr.op, Opcode::Movz);
    assert_eq!(instr.imm, 0x1234);
}

#[test]
fn movz_lsl_16() {
    let instr = decode(0xD2A2_4680).unwrap();
    assert_eq!(instr.imm, 0x1234_0000);
}

#[test]
fn decode_cmp_x3_x2() {
    let instr = decode(0xEB02007F).unwrap();
    assert_eq!(instr.op, Opcode::Cmp);
    assert_eq!(instr.rn, 3);
    assert_eq!(instr.rm, 2);
}

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
}

#[test]
fn decode_adrp_non_zero_immlo() {
    let raw: u32 = 0xf0000d61;
    let instr = decode(raw).unwrap();
    assert_eq!(instr.op, Opcode::Adrp);
    assert_eq!(instr.rd, 1);
    assert_eq!(instr.imm, 0x1af000);
}
