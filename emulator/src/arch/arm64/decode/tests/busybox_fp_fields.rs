use super::*;

#[test]
fn decode_busybox_fp_field_details() {
    let fixed = decode(0x1E42_F800).unwrap();
    assert_eq!(fixed.imm, 2);
    assert_eq!(fixed.cond, 1);

    let fixed_unsigned = decode(0x1E03_FC00).unwrap();
    assert_eq!(fixed_unsigned.imm, 1);
    assert_eq!(fixed_unsigned.cond, 1);

    let fixed_to_signed = decode(0x1E58_FBE0).unwrap();
    assert_eq!(fixed_to_signed.op, Opcode::Fcvtzs);
    assert_eq!(fixed_to_signed.rn, 31);
    assert_eq!(fixed_to_signed.imm, 2);
    assert_eq!(fixed_to_signed.cond, 1);
    assert_eq!(fixed_to_signed.size, 8);
    assert!(!fixed_to_signed.sf);

    let fixed_to_unsigned = decode(0x1E19_F3A1).unwrap();
    assert_eq!(fixed_to_unsigned.op, Opcode::Fcvtzu);
    assert_eq!(fixed_to_unsigned.rd, 1);
    assert_eq!(fixed_to_unsigned.rn, 29);
    assert_eq!(fixed_to_unsigned.imm, 4);
    assert_eq!(fixed_to_unsigned.cond, 1);
    assert_eq!(fixed_to_unsigned.size, 4);
    assert!(!fixed_to_unsigned.sf);
    assert!(decode(0x1E18_03E0).is_none());

    let fmov_five = decode(0x1E62_900F).unwrap();
    assert_eq!(fmov_five.rd, 15);
    assert_eq!(fmov_five.imm, 20);

    let fmov_vec = decode(0x4F03_F61E).unwrap(); // fmov v30.4s, #1
    assert_eq!(fmov_vec.op, Opcode::SimdFmovImm);
    assert_eq!(fmov_vec.rd, 30);
    assert_eq!(fmov_vec.imm, 0x70);
    assert_eq!(fmov_vec.cond, 4);
    assert_eq!(fmov_vec.size, 16);

    let fmov_vec_double = decode(0x6F07_F61F).unwrap(); // fmov v31.2d, #-1
    assert_eq!(fmov_vec_double.op, Opcode::SimdFmovImm);
    assert_eq!(fmov_vec_double.rd, 31);
    assert_eq!(fmov_vec_double.imm, 0xF0);
    assert_eq!(fmov_vec_double.cond, 8);
    assert_eq!(fmov_vec_double.size, 16);
    assert!(decode(0x2F07_F61F).is_none());

    let fmov_single_reg = decode(0x1E20_43DD).unwrap();
    assert_eq!(fmov_single_reg.op, Opcode::SimdFmovReg64);
    assert_eq!(fmov_single_reg.rd, 29);
    assert_eq!(fmov_single_reg.rn, 30);
    assert_eq!(fmov_single_reg.size, 4);

    let frinta = decode(0x1E66_43FF).unwrap();
    assert_eq!(frinta.op, Opcode::FpFrinta);
    assert_eq!(frinta.rd, 31);
    assert_eq!(frinta.rn, 31);
    assert_eq!(frinta.size, 8);

    let fcvtas = decode(0x9E64_03A3).unwrap();
    assert_eq!(fcvtas.op, Opcode::Fcvtas);
    assert_eq!(fcvtas.rd, 3);
    assert_eq!(fcvtas.rn, 29);
    assert_eq!(fcvtas.size, 8);
    assert!(fcvtas.sf);

    let fcvt_double_to_single = decode(0x1E62_401F).unwrap();
    assert_eq!(fcvt_double_to_single.size, 4);
    assert_eq!(fcvt_double_to_single.cond, 8);

    let fcvt_single_to_double = decode(0x1E22_C000).unwrap();
    assert_eq!(fcvt_single_to_double.size, 8);
    assert_eq!(fcvt_single_to_double.cond, 4);

    let fcvt_half_to_single = decode(0x1EE2_4015).unwrap();
    assert_eq!(fcvt_half_to_single.rd, 21);
    assert_eq!(fcvt_half_to_single.rn, 0);
    assert_eq!(fcvt_half_to_single.size, 4);
    assert_eq!(fcvt_half_to_single.cond, 2);

    let fcvt_half_to_double = decode(0x1EE2_C015).unwrap();
    assert_eq!(fcvt_half_to_double.size, 8);
    assert_eq!(fcvt_half_to_double.cond, 2);

    let fcvt_single_to_half = decode(0x1E23_C3DE).unwrap();
    assert_eq!(fcvt_single_to_half.rd, 30);
    assert_eq!(fcvt_single_to_half.rn, 30);
    assert_eq!(fcvt_single_to_half.size, 2);
    assert_eq!(fcvt_single_to_half.cond, 4);

    let fcvt_double_to_half = decode(0x1E63_C39C).unwrap();
    assert_eq!(fcvt_double_to_half.rd, 28);
    assert_eq!(fcvt_double_to_half.rn, 28);
    assert_eq!(fcvt_double_to_half.size, 2);
    assert_eq!(fcvt_double_to_half.cond, 8);

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

    let fmla_elem = decode(0x4FB9_1BEF).unwrap(); // fmla v15.4s, v31.4s, v25.s[3]
    assert_eq!(fmla_elem.rd, 15);
    assert_eq!(fmla_elem.rn, 31);
    assert_eq!(fmla_elem.rm, 25);
    assert_eq!(fmla_elem.cond, 3);
    assert_eq!(fmla_elem.imm, 4);
    assert_eq!(fmla_elem.size, 16);

    let fmul_elem = decode(0x4FC9_9907).unwrap(); // fmul v7.2d, v8.2d, v9.d[1]
    assert_eq!(fmul_elem.op, Opcode::SimdFpMulElem);
    assert_eq!(fmul_elem.rd, 7);
    assert_eq!(fmul_elem.rn, 8);
    assert_eq!(fmul_elem.rm, 9);
    assert_eq!(fmul_elem.cond, 1);
    assert_eq!(fmul_elem.imm, 8);
    assert_eq!(fmul_elem.size, 16);

    let facgt = decode(0x6EBD_EC21).unwrap(); // facgt v1.4s, v1.4s, v29.4s
    assert_eq!(facgt.op, Opcode::SimdFpFacgtVec);
    assert_eq!(facgt.rd, 1);
    assert_eq!(facgt.rn, 1);
    assert_eq!(facgt.rm, 29);
    assert_eq!(facgt.imm, 4);
    assert_eq!(facgt.size, 16);

    let facge = decode(0x6E20_EC21).unwrap(); // facge v1.4s, v1.4s, v0.4s
    assert_eq!(facge.op, Opcode::SimdFpFacgeVec);
    assert_eq!(facge.rd, 1);
    assert_eq!(facge.rn, 1);
    assert_eq!(facge.rm, 0);
    assert_eq!(facge.imm, 4);
    assert_eq!(facge.size, 16);
    assert!(decode(0x2E60_EC21).is_none());
}
