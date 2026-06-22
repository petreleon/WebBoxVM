use crate::arch::arm64::Opcode;

use super::decode;

#[test]
fn decode_observed_daifset_three() {
    let instr = decode(0xd503_43df).expect("decode msr daifset, #3");

    assert_eq!(instr.op, Opcode::DaifSet);
    assert_eq!(instr.cond, 1);
    assert_eq!(instr.imm, 3);
}

#[test]
fn decode_daifclr_three() {
    let instr = decode(0xd503_43ff).expect("decode msr daifclr, #3");

    assert_eq!(instr.op, Opcode::DaifClr);
    assert_eq!(instr.cond, 2);
    assert_eq!(instr.imm, 3);
}
