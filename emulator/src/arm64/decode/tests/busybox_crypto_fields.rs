use super::*;

#[test]
fn decode_busybox_crypto_and_vector_field_details() {
    let fcmp_zero = decode(0x1E60_23E8).unwrap();
    assert_eq!(fcmp_zero.rn, 31);
    assert_eq!(fcmp_zero.cond, 1);

    let fccmp = decode(0x1E6D_05E4).unwrap();
    assert_eq!(fccmp.op, Opcode::Fccmp);
    assert_eq!(fccmp.rn, 15);
    assert_eq!(fccmp.rm, 13);
    assert_eq!(fccmp.imm, 4);
    assert_eq!(fccmp.cond, 0);
    assert_eq!(fccmp.size, 8);

    let fccmp_single = decode(0x1E3F_8400).unwrap();
    assert_eq!(fccmp_single.op, Opcode::Fccmp);
    assert_eq!(fccmp_single.cond, 8);
    assert_eq!(fccmp_single.size, 4);

    let eor3 = decode(0xCE02_0C24).unwrap();
    assert_eq!(eor3.rd, 4);
    assert_eq!(eor3.rn, 1);
    assert_eq!(eor3.rm, 2);
    assert_eq!(eor3.cond, 3);
    assert_eq!(eor3.size, 16);

    let xar = decode(0xCE82_3424).unwrap();
    assert_eq!(xar.rd, 4);
    assert_eq!(xar.rn, 1);
    assert_eq!(xar.rm, 2);
    assert_eq!(xar.imm, 13);
    assert_eq!(xar.size, 16);

    let pmull = decode(0x0EE0_E000).unwrap();
    assert_eq!(pmull.op, Opcode::SimdPmull);
    assert_eq!(pmull.rd, 0);
    assert_eq!(pmull.rn, 0);
    assert_eq!(pmull.rm, 0);
    assert_eq!(pmull.imm, 8);
    assert_eq!(pmull.size, 16);
    assert!(decode(0x0E60_E000).is_none());
    assert!(decode(0x0EA0_E000).is_none());

    let sha1h = decode(0x5E28_0800).unwrap();
    assert_eq!(sha1h.op, Opcode::SimdSha1h);
    assert_eq!(sha1h.rd, 0);
    assert_eq!(sha1h.rn, 0);
    assert_eq!(sha1h.size, 4);

    let sha256su0 = decode(0x5E28_2800).unwrap();
    assert_eq!(sha256su0.op, Opcode::SimdSha256Su0);
    assert_eq!(sha256su0.rd, 0);
    assert_eq!(sha256su0.rn, 0);
    assert_eq!(sha256su0.size, 16);

    let sha512su0 = decode(0xCEC0_8000).unwrap();
    assert_eq!(sha512su0.op, Opcode::SimdSha512Su0);
    assert_eq!(sha512su0.rd, 0);
    assert_eq!(sha512su0.rn, 0);
    assert_eq!(sha512su0.size, 16);

    let sm4e = decode(0xCEC0_8400).unwrap();
    assert_eq!(sm4e.op, Opcode::SimdSm4e);
    assert_eq!(sm4e.rd, 0);
    assert_eq!(sm4e.rn, 0);
    assert_eq!(sm4e.size, 16);

    let sm3partw1 = decode(0xCE63_C004).unwrap();
    assert_eq!(sm3partw1.op, Opcode::SimdSm3Partw1);
    assert_eq!(sm3partw1.rd, 4);
    assert_eq!(sm3partw1.rn, 0);
    assert_eq!(sm3partw1.rm, 3);
    assert_eq!(sm3partw1.size, 16);

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

    let mul = decode(0x0EBE_9FBD).unwrap();
    assert_eq!(mul.op, Opcode::SimdMulVec);
    assert_eq!(mul.rd, 29);
    assert_eq!(mul.rn, 29);
    assert_eq!(mul.rm, 30);
    assert_eq!(mul.imm, 4);
    assert_eq!(mul.size, 8);

    let mul_16b = decode(0x4E20_9C00).unwrap();
    assert_eq!(mul_16b.op, Opcode::SimdMulVec);
    assert_eq!(mul_16b.imm, 1);
    assert_eq!(mul_16b.size, 16);

    let mla = decode(0x4EBF_97FE).unwrap();
    assert_eq!(mla.op, Opcode::SimdMlaVec);
    assert_eq!(mla.rd, 30);
    assert_eq!(mla.rn, 31);
    assert_eq!(mla.rm, 31);
    assert_eq!(mla.imm, 4);
    assert_eq!(mla.size, 16);
    assert!(decode(0x4EE0_9400).is_none());
}
