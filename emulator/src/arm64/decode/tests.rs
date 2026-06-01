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
fn decode_addsub_with_carry() {
    let adc = decode(0x9A02_0020).unwrap(); // adc x0, x1, x2
    assert_eq!(adc.op, Opcode::Adc);
    assert_eq!(adc.rd, 0);
    assert_eq!(adc.rn, 1);
    assert_eq!(adc.rm, 2);

    let sbc = decode(0xDA1F_03E0).unwrap(); // sbc x0, xzr, xzr
    assert_eq!(sbc.op, Opcode::Sbc);
    assert_eq!(sbc.rd, 0);
    assert_eq!(sbc.rn, 31);
    assert_eq!(sbc.rm, 31);
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
fn decode_dc_zva() {
    let instr = decode(0xD50B_7423).unwrap(); // dc zva, x3
    assert_eq!(instr.op, Opcode::DcZva);
    assert_eq!(instr.rd, 3);
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
fn decode_stlxp_pair() {
    let instr = decode(0xC823_8440).unwrap(); // stlxp w3, x0, x1, [x2]
    assert_eq!(instr.op, Opcode::Stxp);
    assert_eq!(instr.rd, 0);
    assert_eq!(instr.rm, 1);
    assert_eq!(instr.rn, 2);
    assert_eq!(instr.imm, 3);
}

#[test]
fn decode_ldpsw_pair() {
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
}

#[test]
fn decode_simd_userland_string_ops() {
    assert_eq!(decode(0x0E04_0E8F).unwrap().op, Opcode::SimdDupByte);
    assert_eq!(decode(0x4E04_0C40).unwrap().op, Opcode::SimdDupByte);
    assert_eq!(decode(0x4EB9_1F18).unwrap().op, Opcode::SimdOrr);
    assert_eq!(decode(0x4E20_1C21).unwrap().op, Opcode::SimdAnd);
    assert_eq!(decode(0x0E20_5BFF).unwrap().op, Opcode::SimdCnt);
    assert_eq!(decode(0x0E31_BBFF).unwrap().op, Opcode::SimdAddv);
    assert_eq!(decode(0x4C40_A03E).unwrap().op, Opcode::SimdLd1Multi);
    assert_eq!(decode(0x4C00_A2DE).unwrap().op, Opcode::SimdSt1Multi);
    let post_index_ld1 = decode(0x4CDF_7A04).unwrap();
    assert_eq!(post_index_ld1.op, Opcode::SimdLd1);
    assert_eq!(post_index_ld1.rd, 4);
    assert_eq!(post_index_ld1.rn, 16);
    assert_eq!(post_index_ld1.imm, 16);
    assert_eq!(post_index_ld1.size, 16);
    assert_eq!(decode(0x4E22_BC45).unwrap().op, Opcode::SimdAddp);
    assert_eq!(decode(0x6E1F_43FF).unwrap().op, Opcode::SimdExt);
    assert_eq!(decode(0x6E20_5BDE).unwrap().op, Opcode::SimdNot);
    assert_eq!(decode(0x4EFE_87FF).unwrap().op, Opcode::SimdAddVec);
    assert_eq!(decode(0x1E26_03E1).unwrap().op, Opcode::SimdFmovSToGpr);
    assert_eq!(decode(0x1E27_009F).unwrap().op, Opcode::SimdFmovGprToS);
    assert_eq!(decode(0x4D40_CC1F).unwrap().op, Opcode::SimdLd1r);
    assert_eq!(decode(0x6F3C_07BD).unwrap().op, Opcode::SimdUshr);
    assert_eq!(decode(0x4EB6_8FDF).unwrap().op, Opcode::SimdCmtst);
    assert_eq!(decode(0x0EA1_2BEF).unwrap().op, Opcode::SimdXtn);
    assert_eq!(decode(0x6E07_079F).unwrap().op, Opcode::SimdInsElem);
    assert_eq!(decode(0x6E20_0BFF).unwrap().op, Opcode::SimdRev32);
    assert_eq!(decode(0x0F2D_57C2).unwrap().op, Opcode::SimdShlImm);
    assert_eq!(decode(0x4E08_077D).unwrap().op, Opcode::SimdDupElem);
    assert_eq!(decode(0x6EF9_47BD).unwrap().op, Opcode::SimdUshl);
    assert_eq!(decode(0x4E9C_1BDE).unwrap().op, Opcode::SimdUzp1);
    assert_eq!(decode(0x0F00_043F).unwrap().op, Opcode::SimdMovi);
    assert_eq!(decode(0x2F00_051E).unwrap().op, Opcode::SimdMvni);
    assert_eq!(decode(0x6E22_AC20).unwrap().op, Opcode::SimdUminp);
    assert_eq!(decode(0x0E20_9800).unwrap().op, Opcode::SimdCmeqZero);
}

#[test]
fn decode_adrp_non_zero_immlo() {
    let raw: u32 = 0xf0000d61;
    let instr = decode(raw).unwrap();
    assert_eq!(instr.op, Opcode::Adrp);
    assert_eq!(instr.rd, 1);
    assert_eq!(instr.imm, 0x1af000);
}
