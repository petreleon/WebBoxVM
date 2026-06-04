use super::*;

#[test]
fn simd_userland_vector_permute_and_reduction_ops() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.set_w(20, 0xaabb_ccdd);
    cpu.regs.set_w(2, 0x1122_3344);

    execute(&mut cpu, &mut bus, decode(0x0E04_0E8F).unwrap()).unwrap(); // dup v15.2s, w20
    assert_eq!(cpu.simd[15], 0xaabb_ccdd_aabb_ccdd);

    execute(&mut cpu, &mut bus, decode(0x4E04_0C40).unwrap()).unwrap(); // dup v0.4s, w2
    assert_eq!(cpu.simd[0], 0x1122_3344_1122_3344_1122_3344_1122_3344);

    cpu.simd[30] = 0x3333_3333_2222_2222_1111_1111_0000_0000;
    execute(&mut cpu, &mut bus, decode(0x5E0C_07DF).unwrap()).unwrap(); // dup s31, v30.s[1]
    assert_eq!(cpu.simd[31], 0x1111_1111);

    execute(&mut cpu, &mut bus, decode(0x5E14_07DF).unwrap()).unwrap(); // dup s31, v30.s[2]
    assert_eq!(cpu.simd[31], 0x2222_2222);

    cpu.simd[20] = 0x8877_6655_4433_2211_0123_4567_89ab_cdef;
    execute(&mut cpu, &mut bus, decode(0x5E18_0694).unwrap()).unwrap(); // dup d20, v20.d[1]
    assert_eq!(cpu.simd[20], 0x8877_6655_4433_2211);

    cpu.simd[24] = 0x4444_4444_3333_3333_2222_2222_1111_1111;
    execute(&mut cpu, &mut bus, decode(0x5E1C_071E).unwrap()).unwrap(); // dup s30, v24.s[3]
    assert_eq!(cpu.simd[30], 0x4444_4444);

    cpu.simd[25] = 0x8888_8888_7777_7777_6666_6666_5555_5555;
    execute(&mut cpu, &mut bus, decode(0x5E1C_073A).unwrap()).unwrap(); // dup s26, v25.s[3]
    assert_eq!(cpu.simd[26], 0x8888_8888);

    cpu.simd[24] = 0x00ff_00ff_00ff_00ff_1111_2222_3333_4444;
    cpu.simd[25] = 0xff00_ff00_ff00_ff00_8888_4444_2222_1111;
    execute(&mut cpu, &mut bus, decode(0x4EB9_1F18).unwrap()).unwrap(); // orr v24.16b, v24.16b, v25.16b
    assert_eq!(cpu.simd[24], 0xffff_ffff_ffff_ffff_9999_6666_3333_5555);

    cpu.simd[0] = 0x00ff_0000_0000_ffff_0123_4567_89ab_cdef;
    cpu.simd[1] = 0xffff_0000_ffff_0000_ffff_ffff_0000_0000;
    execute(&mut cpu, &mut bus, decode(0x4EE1_1C00).unwrap()).unwrap(); // orn v0.16b, v0.16b, v1.16b
    assert_eq!(cpu.simd[0], 0x00ff_ffff_0000_ffff_0123_4567_ffff_ffff);

    cpu.simd[1] = 0xffff_0000_ffff_0000_1234_5678_9abc_def0;
    cpu.simd[0] = 0x0f0f_0f0f_f0f0_f0f0_ffff_0000_ffff_0000;
    execute(&mut cpu, &mut bus, decode(0x4E20_1C21).unwrap()).unwrap(); // and v1.16b, v1.16b, v0.16b
    assert_eq!(cpu.simd[1], 0x0f0f_0000_f0f0_0000_1234_0000_9abc_0000);

    cpu.simd[30] = 0xffff_0000_aaaa_5555_1234_5678_9abc_def0;
    cpu.simd[4] = 0x0000_ffff_0f0f_0f0f_ffff_0000_00ff_00ff;
    execute(&mut cpu, &mut bus, decode(0x0E64_1FDE).unwrap()).unwrap(); // bic v30.8b, v30.8b, v4.8b
    assert_eq!(cpu.simd[30], 0x0000_5678_9a00_de00);

    cpu.simd[1] = 0xffff_ffff_ffff_ffff_00ff_00ff_00ff_00ff;
    cpu.simd[15] = 0x1111_2222_3333_4444;
    cpu.simd[31] = 0xaaaa_bbbb_cccc_dddd;
    execute(&mut cpu, &mut bus, decode(0x2E7F_1DE1).unwrap()).unwrap(); // bsl v1.8b, v15.8b, v31.8b
    assert_eq!(cpu.simd[1], 0xaa11_bb22_cc33_dd44);

    cpu.simd[0] = 0xffff_ffff_ffff_ffff_0123_4567_89ab_cdef;
    cpu.simd[15] = 0xfedc_ba98_7654_3210;
    cpu.simd[31] = 0xffff_0000_ffff_0000;
    execute(&mut cpu, &mut bus, decode(0x2EBF_1DE0).unwrap()).unwrap(); // bit v0.8b, v15.8b, v31.8b
    assert_eq!(cpu.simd[0], 0xfedc_4567_7654_cdef);

    cpu.simd[0] = 0xffff_ffff_ffff_ffff_0123_4567_89ab_cdef;
    cpu.simd[31] = 0xfedc_ba98_7654_3210;
    cpu.simd[30] = 0xffff_0000_ffff_0000;
    execute(&mut cpu, &mut bus, decode(0x2EFE_1FE0).unwrap()).unwrap(); // bif v0.8b, v31.8b, v30.8b
    assert_eq!(cpu.simd[0], 0x0123_ba98_89ab_3210);

    cpu.simd[31] = 0x0102_0304_0506_0708_7f80_55aa_0001_0fff;
    execute(&mut cpu, &mut bus, decode(0x0E20_5BFF).unwrap()).unwrap(); // cnt v31.8b, v31.8b
    execute(&mut cpu, &mut bus, decode(0x0E31_BBFF).unwrap()).unwrap(); // addv b31, v31.8b
    assert_eq!(cpu.simd[31], 29);

    cpu.simd[29] = 0x0000_00fe_0000_1000_ffff_fffe_0000_0007;
    execute(&mut cpu, &mut bus, decode(0x6EB0_ABBF).unwrap()).unwrap(); // umaxv s31, v29.4s
    assert_eq!(cpu.simd[31], 0xffff_fffe);
}
