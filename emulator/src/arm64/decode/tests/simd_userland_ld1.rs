use super::*;

#[test]
fn decode_simd_userland_ld1_forms() {
    assert_eq!(decode(0x4C40_A03E).unwrap().op, Opcode::SimdLd1Multi);
    assert_eq!(decode(0x4C00_A2DE).unwrap().op, Opcode::SimdSt1Multi);
    let ld1r_post_imm = decode(0x4DDF_CD5A).unwrap();
    assert_eq!(ld1r_post_imm.op, Opcode::SimdLd1r);
    assert_eq!(ld1r_post_imm.rd, 26);
    assert_eq!(ld1r_post_imm.rn, 10);
    assert_eq!(ld1r_post_imm.rm, 0xFE);
    assert_eq!(ld1r_post_imm.imm, 8);
    assert_eq!(ld1r_post_imm.cond, 8);
    assert_eq!(ld1r_post_imm.size, 16);
    let ld1r_post_reg = decode(0x4DC1_CC00).unwrap();
    assert_eq!(ld1r_post_reg.op, Opcode::SimdLd1r);
    assert_eq!(ld1r_post_reg.rm, 1);
    assert_eq!(ld1r_post_reg.imm, 0);
    let post_index_ld1 = decode(0x4CDF_7A04).unwrap();
    assert_eq!(post_index_ld1.op, Opcode::SimdLd1);
    assert_eq!(post_index_ld1.rd, 4);
    assert_eq!(post_index_ld1.rn, 16);
    assert_eq!(post_index_ld1.rm, 0xFE);
    assert_eq!(post_index_ld1.imm, 16);
    assert_eq!(post_index_ld1.size, 16);
    let ld1_post_q0 = decode(0x0CDF_7004).unwrap();
    assert_eq!(ld1_post_q0.op, Opcode::SimdLd1);
    assert_eq!(ld1_post_q0.rd, 4);
    assert_eq!(ld1_post_q0.rn, 0);
    assert_eq!(ld1_post_q0.imm, 8);
    assert_eq!(ld1_post_q0.size, 8);
    let ld1_reg_post = decode(0x4CC8_7000).unwrap();
    assert_eq!(ld1_reg_post.op, Opcode::SimdLd1);
    assert_eq!(ld1_reg_post.rm, 8);
    assert_eq!(ld1_reg_post.imm, 0);
    let ld1_one = decode(0x4C40_7840).unwrap();
    assert_eq!(ld1_one.op, Opcode::SimdLd1);
    assert_eq!(ld1_one.rd, 0);
    assert_eq!(ld1_one.rn, 2);
    assert_eq!(ld1_one.size, 16);
    let ld1_two = decode(0x4CDF_A8F6).unwrap();
    assert_eq!(ld1_two.op, Opcode::SimdLd1Multi);
    assert_eq!(ld1_two.rd, 22);
    assert_eq!(ld1_two.rn, 7);
    assert_eq!(ld1_two.cond, 2);
    assert_eq!(ld1_two.imm, 32);
    assert_eq!(ld1_two.size, 16);
}
