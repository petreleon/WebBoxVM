use super::*;

#[test]
fn decode_sve_register_load_store_forms() {
    let cases = [
        (0x8580_43E8, Opcode::SveLdr, "ldr"),
        (0x8580_03E4, Opcode::SveLdr, "ldr"),
        (0xE580_43EA, Opcode::SveStr, "str"),
        (0xE580_03E4, Opcode::SveStr, "str"),
    ];
    for (raw, expected, mnemonic) in cases {
        assert_disarm64_mnemonic(raw, mnemonic);
        assert_eq!(decode(raw).unwrap().op, expected, "raw=0x{raw:08x}");
    }

    let ldr_z = decode(0x8582_4BF7).unwrap(); // ldr z23, [sp, #18, mul vl]
    assert_eq!(ldr_z.op, Opcode::SveLdr);
    assert_eq!(ldr_z.rd, 23);
    assert_eq!(ldr_z.rn, 31);
    assert_eq!(ldr_z.imm as i64, 18);
    assert_eq!(ldr_z.cond, 1);

    let str_z_neg = decode(0xE5BF_5FEA).unwrap(); // str z10, [sp, #-1, mul vl]
    assert_eq!(str_z_neg.op, Opcode::SveStr);
    assert_eq!(str_z_neg.rd, 10);
    assert_eq!(str_z_neg.imm as i64, -1);
    assert_eq!(str_z_neg.cond, 1);

    let ldr_p = decode(0x8581_03EB).unwrap(); // ldr p11, [sp, #8, mul vl]
    assert_eq!(ldr_p.op, Opcode::SveLdr);
    assert_eq!(ldr_p.rd, 11);
    assert_eq!(ldr_p.imm as i64, 8);
    assert_eq!(ldr_p.cond, 0);

    let str_p_neg = decode(0xE5BF_1FE4).unwrap(); // str p4, [sp, #-1, mul vl]
    assert_eq!(str_p_neg.op, Opcode::SveStr);
    assert_eq!(str_p_neg.rd, 4);
    assert_eq!(str_p_neg.imm as i64, -1);
    assert_eq!(str_p_neg.cond, 0);
}

#[test]
fn decode_sve_predicated_dword_load_store_forms() {
    let cases = [
        (0x85C8_EC07, Opcode::SveLd1rd, "ld1rd"),
        (0x8540_C000, Opcode::SveLd1rw, "ld1rw"),
        (0xA582_2C00, Opcode::SveLd1rqd, "ld1rqd"),
        (0xA501_2C00, Opcode::SveLd1rqw, "ld1rqw"),
        (0xC5E6_C07D, Opcode::SveLd1d, "ld1d"),
        (0xA5E0_AC00, Opcode::SveLd1d, "ld1d"),
        (0xA40E_A082, Opcode::SveLd1b, "ld1b"),
        (0xE5E0_EC00, Opcode::SveSt1d, "st1d"),
        (0xE400_E000, Opcode::SveSt1b, "st1b"),
        (0xE420_E000, Opcode::SveSt1b, "st1b"),
        (0xE440_E000, Opcode::SveSt1b, "st1b"),
        (0xE460_E000, Opcode::SveSt1b, "st1b"),
    ];
    for (raw, expected, mnemonic) in cases {
        assert_disarm64_mnemonic(raw, mnemonic);
        assert_eq!(decode(raw).unwrap().op, expected, "raw=0x{raw:08x}");
    }

    let ld1rd = decode(0x85C8_EC07).unwrap(); // ld1rd { z7.d }, p3/z, [x0, #0x40]
    assert_eq!(ld1rd.rd, 7);
    assert_eq!(ld1rd.rn, 0);
    assert_eq!(ld1rd.cond, 3);
    assert_eq!(ld1rd.imm, 64);
    assert_eq!(ld1rd.size, 8);

    let ld1rqd = decode(0xA58E_2C07).unwrap(); // ld1rqd { z7.d }, p3/z, [x0, #-0x20]
    assert_eq!(ld1rqd.rd, 7);
    assert_eq!(ld1rqd.cond, 3);
    assert_eq!(ld1rqd.imm as i64, -32);

    let ld1rqw = decode(0xA50F_2C07).unwrap(); // ld1rqw { z7.s }, p3/z, [x0, #-0x10]
    assert_eq!(ld1rqw.op, Opcode::SveLd1rqw);
    assert_eq!(ld1rqw.rd, 7);
    assert_eq!(ld1rqw.cond, 3);
    assert_eq!(ld1rqw.imm as i64, -16);
    assert_eq!(ld1rqw.size, 4);

    let ld1rw = decode(0x857F_CC07).unwrap(); // ld1rw { z7.s }, p3/z, [x0, #0xfc]
    assert_eq!(ld1rw.op, Opcode::SveLd1rw);
    assert_eq!(ld1rw.imm, 252);
    assert_eq!(ld1rw.size, 4);

    let ld1b = decode(0xA461_ACA7).unwrap(); // ld1b { z7.d }, p3/z, [x5, #0x1, mul vl]
    assert_eq!(ld1b.op, Opcode::SveLd1b);
    assert_eq!(ld1b.rd, 7);
    assert_eq!(ld1b.rn, 5);
    assert_eq!(ld1b.cond, 3);
    assert_eq!(ld1b.imm as i64, 1);
    assert_eq!(ld1b.size, 8);

    let gather = decode(0xC5E6_C07D).unwrap(); // ld1d { z29.d }, p0/z, [x3, z6.d, lsl #3]
    assert_eq!(gather.rd, 29);
    assert_eq!(gather.rn, 3);
    assert_eq!(gather.rm, 6);
    assert_eq!(gather.cond, 0);

    let ld1d_imm = decode(0xA5E1_AC00).unwrap(); // ld1d { z0.d }, p3/z, [x0, #0x1, mul vl]
    assert_eq!(ld1d_imm.rd, 0);
    assert_eq!(ld1d_imm.rn, 0);
    assert_eq!(ld1d_imm.rm, 0xFF);
    assert_eq!(ld1d_imm.cond, 3);
    assert_eq!(ld1d_imm.imm as i64, 1);

    let st1d_imm = decode(0xE5EF_EC00).unwrap(); // st1d { z0.d }, p3, [x0, #-0x1, mul vl]
    assert_eq!(st1d_imm.op, Opcode::SveSt1d);
    assert_eq!(st1d_imm.rm, 0xFF);
    assert_eq!(st1d_imm.imm as i64, -1);

    let st1b = decode(0xE40E_E082).unwrap(); // st1b { z2.b }, p0, [x4, #-0x2, mul vl]
    assert_eq!(st1b.op, Opcode::SveSt1b);
    assert_eq!(st1b.rd, 2);
    assert_eq!(st1b.rn, 4);
    assert_eq!(st1b.cond, 0);
    assert_eq!(st1b.imm as i64, -2);
    assert_eq!(st1b.size, 1);
}
