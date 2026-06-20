use super::*;

#[test]
fn decode_stlxp_pair() {
    assert_disarm64_mnemonic(0xC823_8440, "stlxp");

    let instr = decode(0xC823_8440).unwrap(); // stlxp w3, x0, x1, [x2]
    assert_eq!(instr.op, Opcode::Stxp);
    assert_eq!(instr.rd, 0);
    assert_eq!(instr.rm, 1);
    assert_eq!(instr.rn, 2);
    assert_eq!(instr.imm, 3);
}

#[test]
fn decode_ldpsw_pair() {
    assert_disarm64_mnemonic(0x694C_9262, "ldpsw");

    let instr = decode(0x694C_9262).unwrap(); // ldpsw x2, x4, [x19, #100]
    assert_eq!(instr.op, Opcode::Ldpsw);
    assert_eq!(instr.rd, 2);
    assert_eq!(instr.rm, 4);
    assert_eq!(instr.rn, 19);
    assert_eq!(instr.imm, 100);
    assert_eq!(instr.size, 0);
}

#[test]
fn decode_simd_q_load_store_forms() {
    let stur = decode(0x3C80_82BE).unwrap(); // stur q30, [x21, #8]
    assert_eq!(stur.op, Opcode::SimdStr);
    assert_eq!(stur.rd, 30);
    assert_eq!(stur.rn, 21);
    assert_eq!(stur.imm, 8);
    assert_eq!(stur.size, 16);

    let reg = decode(0x3CA6_68BE).unwrap(); // str q30, [x5, x6]
    assert_eq!(reg.op, Opcode::SimdStr);
    assert_eq!(reg.rn, 5);
    assert_eq!(reg.rm, 6);
    assert_eq!(reg.imm, 0);
    assert_eq!(reg.size, 16);
}

#[test]
fn decode_simd_q_pair_forms() {
    assert_disarm64_mnemonic(0xAD40_70DD, "ldp");
    assert_disarm64_mnemonic(0xAC81_78DF, "stp");

    let ldp = decode(0xAD40_70DD).unwrap(); // ldp q29, q28, [x6]
    assert_eq!(ldp.op, Opcode::SimdLdp);
    assert_eq!(ldp.rd, 29);
    assert_eq!(ldp.rm, 28);
    assert_eq!(ldp.rn, 6);
    assert_eq!(ldp.imm, 0);
    assert_eq!(ldp.size, 16);

    let stp_post = decode(0xAC81_78DF).unwrap(); // stp q31, q30, [x6], #32
    assert_eq!(stp_post.op, Opcode::SimdStp);
    assert_eq!(stp_post.rd, 31);
    assert_eq!(stp_post.rm, 30);
    assert_eq!(stp_post.rn, 6);
    assert_eq!(stp_post.imm, 32);
    assert_eq!(stp_post.cond, 1);
    assert_eq!(stp_post.size, 16);
}

#[test]
fn decode_simd_zero_and_all_ones_immediates() {
    let movi = decode(0x4F00_041F).unwrap(); // movi v31.4s, #0
    assert_eq!(movi.op, Opcode::SimdMovi);
    assert_eq!(movi.rd, 31);
    assert_eq!(movi.imm, 0);
    assert_eq!(movi.size, 16);

    let mvni = decode(0x6F00_041F).unwrap(); // mvni v31.4s, #0
    assert_eq!(mvni.op, Opcode::SimdMovi);
    assert_eq!(mvni.rd, 31);
    assert_eq!(mvni.imm, u64::MAX);
    assert_eq!(mvni.size, 16);
}

#[test]
fn decode_simd_bic_immediate_does_not_alias_pair_store() {
    let bic = decode(0x6F07_9604).unwrap(); // bic v4.8h, #0xf0
    assert_eq!(bic.op, Opcode::SimdBicImm);
    assert_eq!(bic.rd, 4);
    assert_eq!(bic.imm, 0xf0);
    assert_eq!(bic.cond, 2);
    assert_eq!(bic.size, 16);

    let shifted = decode(0x6F00_B5E2).unwrap(); // bic v2.8h, #0xf, lsl #8
    assert_eq!(shifted.op, Opcode::SimdBicImm);
    assert_eq!(shifted.rd, 2);
    assert_eq!(shifted.imm, 0x0f00);
    assert_eq!(shifted.cond, 2);
    assert_eq!(shifted.size, 16);
}

#[test]
fn decode_simd_umov_halfword() {
    let instr = decode(0x0E02_3FC0).unwrap(); // umov w0, v30.h[0]
    assert_eq!(instr.op, Opcode::SimdUmov);
    assert_eq!(instr.rd, 0);
    assert_eq!(instr.rn, 30);
    assert_eq!(instr.imm, 0);
    assert_eq!(instr.cond, 2);
    assert!(!instr.sf);

    let smov_half = decode(0x0E0A_2FE2).unwrap(); // smov w2, v31.h[2]
    assert_eq!(smov_half.op, Opcode::SimdSmov);
    assert_eq!(smov_half.rd, 2);
    assert_eq!(smov_half.rn, 31);
    assert_eq!(smov_half.imm, 2);
    assert_eq!(smov_half.cond, 2);
    assert!(!smov_half.sf);

    let smov_word_to_x = decode(0x4E04_2FE0).unwrap(); // smov x0, v31.s[0]
    assert_eq!(smov_word_to_x.op, Opcode::SimdSmov);
    assert_eq!(smov_word_to_x.cond, 4);
    assert!(smov_word_to_x.sf);
    assert!(decode(0x0E04_2FE0).is_none());
    assert!(decode(0x4E08_2FE0).is_none());
}
