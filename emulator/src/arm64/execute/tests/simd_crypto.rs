use super::simd_helpers::*;
use super::*;

#[test]
fn simd_aes_crypto_round_ops() {
    let (mut cpu, mut bus) = setup();
    let key = 0x9c95_8e87_8079_726b_645d_564f_4841_3a33;

    cpu.simd[6] = vector_bytes(0);
    cpu.simd[0] = key;
    execute(&mut cpu, &mut bus, decode(0x4E28_4806).unwrap()).unwrap(); // aese v6.16b, v0.16b
    assert_eq!(cpu.simd[6], 0x3d39_e23d_fb1a_ecfb_b314_21b3_dc8f_edc3);

    cpu.simd[2] = vector_bytes(0);
    cpu.simd[0] = key;
    execute(&mut cpu, &mut bus, decode(0x4E28_5802).unwrap()).unwrap(); // aesd v2.16b, v0.16b
    assert_eq!(cpu.simd[2], 0xcc57_03ce_2264_5000_cee8_49cc_008f_4166);

    cpu.simd[2] = vector_bytes(0);
    execute(&mut cpu, &mut bus, decode(0x4E28_6842).unwrap()).unwrap(); // aesmc v2.16b, v2.16b
    assert_eq!(cpu.simd[2], 0x090c_0b0e_0d08_0f0a_0104_0306_0500_0702);

    cpu.simd[0] = cpu.simd[2];
    execute(&mut cpu, &mut bus, decode(0x4E28_7800).unwrap()).unwrap(); // aesimc v0.16b, v0.16b
    assert_eq!(cpu.simd[0], vector_bytes(0));
}

#[test]
fn simd_sha3_bitwise_ops() {
    let (mut cpu, mut bus) = setup();

    cpu.simd[1] = 0x0123_4567_89ab_cdef_fedc_ba98_7654_3210;
    cpu.simd[2] = 0xf0f1_f2f3_f4f5_f6f7_0809_0a0b_0c0d_0e0f;
    cpu.simd[3] = 0x1111_2222_3333_4444_5555_6666_7777_8888;
    execute(&mut cpu, &mut bus, decode(0xCE02_0C24).unwrap()).unwrap(); // eor3 v4.16b, v1.16b, v2.16b, v3.16b
    assert_eq!(cpu.simd[4], 0xe0c3_95b6_4e6d_7f5c_a380_d6f5_0d2e_b497);

    execute(&mut cpu, &mut bus, decode(0xCE22_0C24).unwrap()).unwrap(); // bcax v4.16b, v1.16b, v2.16b, v3.16b
    assert_eq!(cpu.simd[4], 0xe1c3_95b6_4d6f_7f5c_f6d4_b291_7e5c_3417);

    execute(&mut cpu, &mut bus, decode(0xCE62_8C24).unwrap()).unwrap(); // rax1 v4.2d, v1.2d, v2.2d
    assert_eq!(cpu.simd[4], 0xe0c0_a080_6040_2000_eece_ae8e_6e4e_2e0e);

    execute(&mut cpu, &mut bus, decode(0xCE82_3424).unwrap()).unwrap(); // xar v4.2d, v1.2d, v2.2d, #13
    assert_eq!(cpu.simd[4], 0xd8c7_8e95_bca3_eaf1_e0ff_b6ad_849b_d2c9);
}

#[test]
fn simd_crypto_schedule_and_polynomial_ops() {
    let (mut cpu, mut bus) = setup();

    cpu.simd[0] = 0xfedc_ba98_7654_3210_0123_4567_89ab_cdef;
    execute(&mut cpu, &mut bus, decode(0x0EE0_E000).unwrap()).unwrap(); // pmull v0.1q, v0.1d, v0.1d
    assert_eq!(
        cpu.simd[0],
        polynomial_mult_u64(0x0123_4567_89ab_cdef, 0x0123_4567_89ab_cdef)
    );

    cpu.simd[0] = 0xffff_ffff_aaaa_5555_1234_5678_89ab_cdef;
    execute(&mut cpu, &mut bus, decode(0x5E28_0800).unwrap()).unwrap(); // sha1h s0, s0
    assert_eq!(cpu.simd[0], 0x89ab_cdefu32.rotate_left(30) as u128);

    let sha256_input = u32x4([0x0302_0100, 0x0706_0504, 0x0b0a_0908, 0x0f0e_0d0c]);
    cpu.simd[0] = sha256_input;
    execute(&mut cpu, &mut bus, decode(0x5E28_2800).unwrap()).unwrap(); // sha256su0 v0.4s, v0.4s
    assert_eq!(cpu.simd[0], sha256su0_expected(sha256_input, sha256_input));

    let sha512_input = u64x2([0x0123_4567_89ab_cdef, 0xfedc_ba98_7654_3210]);
    cpu.simd[0] = sha512_input;
    execute(&mut cpu, &mut bus, decode(0xCEC0_8000).unwrap()).unwrap(); // sha512su0 v0.2d, v0.2d
    assert_eq!(cpu.simd[0], sha512su0_expected(sha512_input, sha512_input));

    cpu.simd[0] = u32x4([0x0123_4567, 0x89ab_cdef, 0xfedc_ba98, 0x7654_3210]);
    cpu.simd[1] = u32x4([0xf121_86f9, 0x4166_2b61, 0x5a6a_b19a, 0x7ba9_2077]);
    execute(&mut cpu, &mut bus, decode(0xCEC0_8420).unwrap()).unwrap(); // sm4e v0.4s, v1.4s
    assert_eq!(
        cpu.simd[0],
        u32x4([0x27fa_d345, 0xa18b_4cb2, 0x11c1_e22a, 0xcc13_e2ee])
    );

    let sm3_dst = u32x4([0x0011_2233, 0x4455_6677, 0x8899_aabb, 0xccdd_eeff]);
    let sm3_n = u32x4([0x1020_3040, 0x5060_7080, 0x90a0_b0c0, 0xd0e0_f000]);
    let sm3_m = u32x4([0x89ab_cdef, 0x0123_4567, 0x7654_3210, 0xfedc_ba98]);
    cpu.simd[4] = sm3_dst;
    cpu.simd[0] = sm3_n;
    cpu.simd[3] = sm3_m;
    execute(&mut cpu, &mut bus, decode(0xCE63_C004).unwrap()).unwrap(); // sm3partw1 v4.4s, v0.4s, v3.4s
    assert_eq!(cpu.simd[4], sm3partw1_expected(sm3_dst, sm3_n, sm3_m));
}
