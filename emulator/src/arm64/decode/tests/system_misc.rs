use super::*;

#[test]
fn decode_dc_zva() {
    let instr = decode(0xD50B_7423).unwrap(); // dc zva, x3
    assert_eq!(instr.op, Opcode::DcZva);
    assert_eq!(instr.rd, 3);
}

#[test]
fn decode_dmb_ish_as_barrier() {
    let instr = decode(0xD503_3BBF).unwrap(); // dmb ish
    assert_eq!(instr.op, Opcode::NopBarrier);

    let load_barrier = decode(0xD503_39BF).unwrap(); // dmb ishld
    assert_eq!(load_barrier.op, Opcode::NopBarrier);
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
fn decode_prfm_register_offset_as_hint() {
    let instr = decode(0xF8A0_6AB0).unwrap(); // prfm pldl1keep, [x21, x0]
    assert_eq!(instr.op, Opcode::Nop);
}

#[test]
fn decode_daifset_and_daifclr_immediate_fields() {
    let set = decode(0xD503_42DF).unwrap(); // msr daifset, #2
    assert_eq!(set.op, Opcode::Nop);
    assert_eq!(set.cond, 1);
    assert_eq!(set.imm, 2);

    let clear = decode(0xD503_42FF).unwrap(); // msr daifclr, #2
    assert_eq!(clear.op, Opcode::Nop);
    assert_eq!(clear.cond, 2);
    assert_eq!(clear.imm, 2);
}

#[test]
fn decode_extract_separately_from_bitfield() {
    let ror = decode(0x1381_0820).unwrap(); // ror w0, w1, #2
    assert_eq!(ror.op, Opcode::Extr);
    assert_eq!(ror.rd, 0);
    assert_eq!(ror.rn, 1);
    assert_eq!(ror.rm, 1);
    assert_eq!(ror.imm, 2);

    let extr = decode(0x1384_1C62).unwrap(); // extr w2, w3, w4, #7
    assert_eq!(extr.op, Opcode::Extr);
    assert_eq!(extr.rd, 2);
    assert_eq!(extr.rn, 3);
    assert_eq!(extr.rm, 4);
    assert_eq!(extr.imm, 7);
}

#[test]
fn decode_register_rotate_right() {
    assert_disarm64_mnemonic(0x9AC2_2C20, "rorv");

    let rorv = decode(0x9AC2_2C20).unwrap();
    assert_eq!(rorv.op, Opcode::Rorv);
    assert_eq!(rorv.rd, 0);
    assert_eq!(rorv.rn, 1);
    assert_eq!(rorv.rm, 2);
    assert!(rorv.sf);
}
