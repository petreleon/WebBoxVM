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
fn decode_lse_caspal() {
    let instr = decode(0x4860_FC82).unwrap(); // caspal x0, x1, x2, x3, [x4]
    assert_eq!(instr.op, Opcode::Casp);
    assert_eq!(instr.rd, 0);
    assert_eq!(instr.rm, 2);
    assert_eq!(instr.rn, 4);
    assert_eq!(instr.size, 8);
}

#[test]
fn decode_lse_ldaddal() {
    let instr = decode(0xB8E1_0001).unwrap(); // ldaddal w1, w1, [x0]
    assert_eq!(instr.op, Opcode::Atomic);
    assert_eq!(instr.rd, 1);
    assert_eq!(instr.rm, 1);
    assert_eq!(instr.rn, 0);
    assert_eq!(instr.imm, 0);
    assert_eq!(instr.size, 4);
}

#[test]
fn decode_register_offset_str_not_lse_atomic() {
    let instr = decode(0xF82A_780C).unwrap(); // str x12, [x0, x10, lsl #3]
    assert_eq!(instr.op, Opcode::Str);
    assert_eq!(instr.rd, 12);
    assert_eq!(instr.rn, 0);
    assert_eq!(instr.rm, 10);
}

#[test]
fn decode_stlxp_pair() {
    let instr = decode(0xC823_8440).unwrap(); // stlxp w3, x0, x1, [x2]
    assert_eq!(instr.op, Opcode::Stxp);
    assert_eq!(instr.rd, 0);
    assert_eq!(instr.rm, 1);
    assert_eq!(instr.rn, 2);
    assert_eq!(instr.imm, 3);
}

#[test]
fn decode_adrp_non_zero_immlo() {
    let raw: u32 = 0xf0000d61;
    let instr = decode(raw).unwrap();
    assert_eq!(instr.op, Opcode::Adrp);
    assert_eq!(instr.rd, 1);
    assert_eq!(instr.imm, 0x1af000);
}
