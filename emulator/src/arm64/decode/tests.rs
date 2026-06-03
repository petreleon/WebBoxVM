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
fn decode_crc32_scalar_forms() {
    let cases = [
        (0x1AC5_4042, 1, "crc32b"),
        (0x1AC5_4442, 2, "crc32h"),
        (0x1AC3_4842, 4, "crc32w"),
        (0x9AC4_4C42, 8, "crc32x"),
    ];

    for (raw, size, mnemonic) in cases {
        assert_disarm64_mnemonic(raw, mnemonic);
        let instr = decode(raw).unwrap();
        assert_eq!(instr.op, Opcode::Crc32, "raw=0x{raw:08x}");
        assert_eq!(instr.rd, 2);
        assert_eq!(instr.rn, 2);
        assert_eq!(instr.size, size);
    }

    assert_eq!(decode(0x1AC5_4042).unwrap().rm, 5);
    assert_eq!(decode(0x1AC3_4842).unwrap().rm, 3);
    assert_eq!(decode(0x9AC4_4C42).unwrap().rm, 4);
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
    let bic_bytes = decode(0x0E64_1FDE).unwrap();
    assert_eq!(bic_bytes.op, Opcode::SimdBic);
    assert_eq!(bic_bytes.rd, 30);
    assert_eq!(bic_bytes.rn, 30);
    assert_eq!(bic_bytes.rm, 4);
    assert_eq!(bic_bytes.size, 8);
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
    let post_index_st1 = decode(0x4C9F_AA24).unwrap();
    assert_eq!(post_index_st1.op, Opcode::SimdSt1Multi);
    assert_eq!(post_index_st1.rd, 4);
    assert_eq!(post_index_st1.rn, 17);
    assert_eq!(post_index_st1.imm, 32);
    assert_eq!(post_index_st1.cond, 2);
    assert_eq!(post_index_st1.size, 16);
    let post_index_st4 = decode(0x4C9F_7A24).unwrap();
    assert_eq!(post_index_st4.op, Opcode::SimdSt4Single);
    assert_eq!(post_index_st4.rd, 4);
    assert_eq!(post_index_st4.rn, 17);
    assert_eq!(post_index_st4.imm, 16);
    assert_eq!(post_index_st4.cond, 2);
    assert_eq!(post_index_st4.size, 16);
    let ld1_lane_byte = decode(0x0D40_0C7E).unwrap();
    assert_eq!(ld1_lane_byte.op, Opcode::SimdLd1Lane);
    assert_eq!(ld1_lane_byte.rd, 30);
    assert_eq!(ld1_lane_byte.rn, 3);
    assert_eq!(ld1_lane_byte.imm, 3);
    assert_eq!(ld1_lane_byte.cond, 1);
    assert_eq!(ld1_lane_byte.size, 1);
    let st1_lane_half = decode(0x0D00_527C).unwrap();
    assert_eq!(st1_lane_half.op, Opcode::SimdSt1Lane);
    assert_eq!(st1_lane_half.rd, 28);
    assert_eq!(st1_lane_half.rn, 19);
    assert_eq!(st1_lane_half.imm, 2);
    assert_eq!(st1_lane_half.cond, 2);
    assert_eq!(st1_lane_half.size, 2);
    let ld1_lane_double = decode(0x4D40_8400).unwrap();
    assert_eq!(ld1_lane_double.op, Opcode::SimdLd1Lane);
    assert_eq!(ld1_lane_double.imm, 1);
    assert_eq!(ld1_lane_double.cond, 8);
    let ld4 = decode(0x4C40_003C).unwrap();
    assert_eq!(ld4.op, Opcode::SimdLd4);
    assert_eq!(ld4.rd, 28);
    assert_eq!(ld4.rn, 1);
    assert_eq!(ld4.cond, 0);
    assert_eq!(ld4.size, 16);
    assert_eq!(decode(0x4E22_BC45).unwrap().op, Opcode::SimdAddp);
    assert_eq!(decode(0x6E1F_43FF).unwrap().op, Opcode::SimdExt);
    assert_eq!(decode(0x6E20_5BDE).unwrap().op, Opcode::SimdNot);
    assert_eq!(decode(0x4EFE_87FF).unwrap().op, Opcode::SimdAddVec);
    assert_eq!(decode(0x1E26_03E1).unwrap().op, Opcode::SimdFmovSToGpr);
    assert_eq!(decode(0x1E27_009F).unwrap().op, Opcode::SimdFmovGprToS);
    let fmov_lane_insert = decode(0x9EAF_0060).unwrap();
    assert_eq!(fmov_lane_insert.op, Opcode::SimdInsGprLane);
    assert_eq!(fmov_lane_insert.rd, 0);
    assert_eq!(fmov_lane_insert.rn, 3);
    assert_eq!(fmov_lane_insert.imm, 1);
    assert_eq!(decode(0x4D40_CC1F).unwrap().op, Opcode::SimdLd1r);
    assert_eq!(decode(0x6F3C_07BD).unwrap().op, Opcode::SimdUshr);
    assert_eq!(decode(0x4EB6_8FDF).unwrap().op, Opcode::SimdCmtst);
    assert_eq!(decode(0x0EA1_2BEF).unwrap().op, Opcode::SimdXtn);
    assert_eq!(decode(0x6E07_079F).unwrap().op, Opcode::SimdInsElem);
    assert_eq!(decode(0x6E20_0BFF).unwrap().op, Opcode::SimdRev32);
    let rev64 = decode(0x0EA0_0BDE).unwrap();
    assert_eq!(rev64.op, Opcode::SimdRev64);
    assert_eq!(rev64.rd, 30);
    assert_eq!(rev64.rn, 30);
    assert_eq!(rev64.imm, 4);
    assert_eq!(rev64.size, 8);
    assert_eq!(decode(0x0F2D_57C2).unwrap().op, Opcode::SimdShlImm);
    let sli = decode(0x6F39_5486).unwrap();
    assert_eq!(sli.op, Opcode::SimdSli);
    assert_eq!(sli.rd, 6);
    assert_eq!(sli.rn, 4);
    assert_eq!(sli.imm, 25);
    assert_eq!(sli.cond, 4);
    assert_eq!(sli.size, 16);
    assert_eq!(decode(0x4E08_077D).unwrap().op, Opcode::SimdDupElem);
    assert_eq!(decode(0x6EF9_47BD).unwrap().op, Opcode::SimdUshl);
    assert_eq!(decode(0x4E9C_1BDE).unwrap().op, Opcode::SimdUzp1);
    let zip1_bytes = decode(0x4E1B_3BFD).unwrap();
    assert_eq!(zip1_bytes.op, Opcode::SimdZip1);
    assert_eq!(zip1_bytes.rd, 29);
    assert_eq!(zip1_bytes.rn, 31);
    assert_eq!(zip1_bytes.rm, 27);
    assert_eq!(zip1_bytes.imm, 1);
    let zip2_halfwords = decode(0x4E5B_7BFF).unwrap();
    assert_eq!(zip2_halfwords.op, Opcode::SimdZip2);
    assert_eq!(zip2_halfwords.rd, 31);
    assert_eq!(zip2_halfwords.rn, 31);
    assert_eq!(zip2_halfwords.rm, 27);
    assert_eq!(zip2_halfwords.imm, 2);
    let tbl = decode(0x4E17_03FF).unwrap();
    assert_eq!(tbl.op, Opcode::SimdTbl);
    assert_eq!(tbl.rd, 31);
    assert_eq!(tbl.rn, 31);
    assert_eq!(tbl.rm, 23);
    assert_eq!(tbl.cond, 1);
    assert_eq!(tbl.size, 16);
    let simd_fcvtzu = decode(0x7EE1_B800).unwrap();
    assert_eq!(simd_fcvtzu.op, Opcode::SimdFcvtzu);
    assert_eq!(simd_fcvtzu.rd, 0);
    assert_eq!(simd_fcvtzu.rn, 0);
    assert_eq!(simd_fcvtzu.size, 8);
    assert_eq!(decode(0x0F00_043F).unwrap().op, Opcode::SimdMovi);
    let movi_doubleword = decode(0x2F07_E61F).unwrap();
    assert_eq!(movi_doubleword.op, Opcode::SimdMovi);
    assert_eq!(movi_doubleword.rd, 31);
    assert_eq!(movi_doubleword.imm, 0xffff_ffff_0000_0000);
    assert_eq!(movi_doubleword.cond, 8);
    assert_eq!(movi_doubleword.size, 8);
    let movi_half = decode(0x0F00_848F).unwrap();
    assert_eq!(movi_half.op, Opcode::SimdMovi);
    assert_eq!(movi_half.rd, 15);
    assert_eq!(movi_half.imm, 4);
    assert_eq!(movi_half.cond, 2);
    let movi_half_shift = decode(0x0F04_A41F).unwrap();
    assert_eq!(movi_half_shift.op, Opcode::SimdMovi);
    assert_eq!(movi_half_shift.imm, 0x8000);
    assert_eq!(movi_half_shift.cond, 2);
    assert_eq!(decode(0x2F00_051E).unwrap().op, Opcode::SimdMvni);
    assert_eq!(decode(0x6E22_AC20).unwrap().op, Opcode::SimdUminp);
    assert_eq!(decode(0x0E20_9800).unwrap().op, Opcode::SimdCmeqZero);
    let cmeq_words = decode(0x6EB9_8FFF).unwrap();
    assert_eq!(cmeq_words.op, Opcode::SimdCmeqReg);
    assert_eq!(cmeq_words.rd, 31);
    assert_eq!(cmeq_words.rn, 31);
    assert_eq!(cmeq_words.rm, 25);
    assert_eq!(cmeq_words.imm, 4);
    assert_eq!(cmeq_words.size, 16);
    let uqsub = decode(0x7E6F_2FFF).unwrap();
    assert_eq!(uqsub.op, Opcode::SimdUqsub);
    assert_eq!(uqsub.rd, 31);
    assert_eq!(uqsub.rn, 31);
    assert_eq!(uqsub.rm, 15);
    assert_eq!(uqsub.imm, 2);
    assert_eq!(uqsub.size, 2);
    let shll = decode(0x2E21_3BDE).unwrap();
    assert_eq!(shll.op, Opcode::SimdShll);
    assert_eq!(shll.rd, 30);
    assert_eq!(shll.rn, 30);
    assert_eq!(shll.imm, 8);
    assert_eq!(shll.cond, 1);
    assert!(!shll.sf);
    let shll2 = decode(0x6E61_3A21).unwrap();
    assert_eq!(shll2.op, Opcode::SimdShll);
    assert_eq!(shll2.rd, 1);
    assert_eq!(shll2.rn, 17);
    assert_eq!(shll2.imm, 16);
    assert_eq!(shll2.cond, 2);
    assert!(shll2.sf);
    let ssubw = decode(0x0E7E_33BD).unwrap();
    assert_eq!(ssubw.op, Opcode::SimdSsubw);
    assert_eq!(ssubw.rd, 29);
    assert_eq!(ssubw.rn, 29);
    assert_eq!(ssubw.rm, 30);
    assert_eq!(ssubw.cond, 2);
    assert!(!ssubw.sf);
    let ssubw2 = decode(0x4E7E_33FF).unwrap();
    assert_eq!(ssubw2.op, Opcode::SimdSsubw);
    assert_eq!(ssubw2.rd, 31);
    assert_eq!(ssubw2.rn, 31);
    assert_eq!(ssubw2.rm, 30);
    assert_eq!(ssubw2.cond, 2);
    assert!(ssubw2.sf);
    assert_eq!(decode(0x6EE0_FBFF).unwrap().op, Opcode::SimdFpNeg);
    assert_eq!(decode(0x6EFD_87FF).unwrap().op, Opcode::SimdSubVec);
}

#[test]
fn decode_busybox_fp_and_widening_ops_cross_checked_with_disarm64() {
    let cases = [
        (0x1E7F_0800, Opcode::FpMul, "fmul"),
        (0x1E79_2BFF, Opcode::FpAdd, "fadd"),
        (0x1E7B_3B9A, Opcode::FpSub, "fsub"),
        (0x1E60_1BE0, Opcode::FpDiv, "fdiv"),
        (0x1E61_401F, Opcode::FpNeg, "fneg"),
        (0x1E60_C000, Opcode::FpAbs, "fabs"),
        (0x1E61_C000, Opcode::FpSqrt, "fsqrt"),
        (0x1E62_401F, Opcode::FpFcvt, "fcvt"),
        (0x1E22_C000, Opcode::FpFcvt, "fcvt"),
        (0x1E65_4000, Opcode::FpFrintm, "frintm"),
        (0x1E6E_1000, Opcode::FpMovImm, "fmov"),
        (0x1E62_900F, Opcode::FpMovImm, "fmov"),
        (0x1F5B_7B9E, Opcode::Fmadd, "fmadd"),
        (0x1F5F_FBBE, Opcode::Fmsub, "fmsub"),
        (0x1F76_E7F9, Opcode::Fnmsub, "fnmsub"),
        (0x1E62_0000, Opcode::Scvtf, "scvtf"),
        (0x1E22_0280, Opcode::Scvtf, "scvtf"),
        (0x1E42_F800, Opcode::Scvtf, "scvtf"),
        (0x1E63_003F, Opcode::Ucvtf, "ucvtf"),
        (0x9E63_001F, Opcode::Ucvtf, "ucvtf"),
        (0x1E03_FC00, Opcode::Ucvtf, "ucvtf"),
        (0x1E78_03E0, Opcode::Fcvtzs, "fcvtzs"),
        (0x1E79_03E0, Opcode::Fcvtzu, "fcvtzu"),
        (0x9E79_0000, Opcode::Fcvtzu, "fcvtzu"),
        (0x1E60_23E8, Opcode::Fcmp, "fcmp"),
        (0x1E79_23B0, Opcode::Fcmpe, "fcmpe"),
        (0x1E7E_0FFF, Opcode::Fcsel, "fcsel"),
        (0x6F00_E400, Opcode::SimdMovi, "movi"),
        (0x0F00_848F, Opcode::SimdMovi, "movi"),
        (0x0F04_A41F, Opcode::SimdMovi, "movi"),
        (0x2F00_E41F, Opcode::SimdMovi, "movi"),
        (0x2F07_E61F, Opcode::SimdMovi, "movi"),
        (0x2F20_A7FF, Opcode::SimdUshll, "ushll"),
        (0x0F20_A7FF, Opcode::SimdSshll, "sshll"),
        (0x2E21_3BDE, Opcode::SimdShll, "shll"),
        (0x6E21_3BD0, Opcode::SimdShll, "shll2"),
        (0x0E7E_33BD, Opcode::SimdSsubw, "ssubw"),
        (0x4E7E_33FF, Opcode::SimdSsubw, "ssubw2"),
        (0x6EB9_8FFF, Opcode::SimdCmeqReg, "cmeq"),
        (0x7E6F_2FFF, Opcode::SimdUqsub, "uqsub"),
        (0x4E1B_3BFD, Opcode::SimdZip1, "zip1"),
        (0x4E5B_7BFF, Opcode::SimdZip2, "zip2"),
        (0x4E17_03FF, Opcode::SimdTbl, "tbl"),
        (0x7EE1_B800, Opcode::SimdFcvtzu, "fcvtzu"),
        (0x0EA0_0BDE, Opcode::SimdRev64, "rev64"),
    ];

    for (raw, expected, mnemonic) in cases {
        assert_disarm64_mnemonic(raw, mnemonic);
        assert_eq!(decode(raw).unwrap().op, expected, "raw=0x{raw:08x}");
    }

    let fixed = decode(0x1E42_F800).unwrap();
    assert_eq!(fixed.imm, 2);
    assert_eq!(fixed.cond, 1);

    let fixed_unsigned = decode(0x1E03_FC00).unwrap();
    assert_eq!(fixed_unsigned.imm, 1);
    assert_eq!(fixed_unsigned.cond, 1);

    let fmov_five = decode(0x1E62_900F).unwrap();
    assert_eq!(fmov_five.rd, 15);
    assert_eq!(fmov_five.imm, 20);

    let fmov_single_reg = decode(0x1E20_43DD).unwrap();
    assert_eq!(fmov_single_reg.op, Opcode::SimdFmovReg64);
    assert_eq!(fmov_single_reg.rd, 29);
    assert_eq!(fmov_single_reg.rn, 30);
    assert_eq!(fmov_single_reg.size, 4);

    let fcvt_double_to_single = decode(0x1E62_401F).unwrap();
    assert_eq!(fcvt_double_to_single.size, 4);
    assert_eq!(fcvt_double_to_single.cond, 8);

    let fcvt_single_to_double = decode(0x1E22_C000).unwrap();
    assert_eq!(fcvt_single_to_double.size, 8);
    assert_eq!(fcvt_single_to_double.cond, 4);

    let fmadd = decode(0x1F5B_7B9E).unwrap();
    assert_eq!(fmadd.rd, 30);
    assert_eq!(fmadd.rn, 28);
    assert_eq!(fmadd.rm, 27);
    assert_eq!(fmadd.cond, 30);

    let fmsub = decode(0x1F5F_FBBE).unwrap();
    assert_eq!(fmsub.rd, 30);
    assert_eq!(fmsub.rn, 29);
    assert_eq!(fmsub.rm, 31);
    assert_eq!(fmsub.cond, 30);

    let fcmp_zero = decode(0x1E60_23E8).unwrap();
    assert_eq!(fcmp_zero.rn, 31);
    assert_eq!(fcmp_zero.cond, 1);

    let ushll = decode(0x2F20_A7FF).unwrap();
    assert_eq!(ushll.rd, 31);
    assert_eq!(ushll.rn, 31);
    assert_eq!(ushll.imm, 0);
    assert_eq!(ushll.cond, 4);

    let sshll = decode(0x0F20_A7FF).unwrap();
    assert_eq!(sshll.rd, 31);
    assert_eq!(sshll.rn, 31);
    assert_eq!(sshll.imm, 0);
    assert_eq!(sshll.cond, 4);
}

#[test]
fn decode_adrp_non_zero_immlo() {
    let raw: u32 = 0xf0000d61;
    let instr = decode(raw).unwrap();
    assert_eq!(instr.op, Opcode::Adrp);
    assert_eq!(instr.rd, 1);
    assert_eq!(instr.imm, 0x1af000);
}

fn assert_disarm64_mnemonic(raw: u32, mnemonic: &str) {
    let decoded = disarm64::decoder::decode(raw).expect("disarm64 should decode test word");
    assert_eq!(format!("{:?}", decoded.mnemonic), mnemonic);
}
