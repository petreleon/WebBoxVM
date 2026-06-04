use super::simd_helpers::*;
use super::*;

#[test]
fn simd_userland_shift_and_insert_ops() {
    let (mut cpu, mut bus) = setup();

    cpu.simd[29] = 0x1234_5678_0000_0010_8000_0000_ffff_fff0;
    execute(&mut cpu, &mut bus, decode(0x6F3C_07BD).unwrap()).unwrap(); // ushr v29.4s, v29.4s, #4
    assert_eq!(cpu.simd[29], 0x0123_4567_0000_0001_0800_0000_0fff_ffff);

    cpu.simd[31] = 0x99aa_bbcc_ddee_ff00_1122_3344_5566_7788;
    execute(&mut cpu, &mut bus, decode(0x0EA1_2BEF).unwrap()).unwrap(); // xtn v15.2s, v31.2d
    assert_eq!(cpu.simd[15], 0xddee_ff00_5566_7788);

    cpu.simd[31] = 0x0011_2233_4455_6677_8899_aabb_ccdd_eeff;
    cpu.simd[28] = 0xaa;
    execute(&mut cpu, &mut bus, decode(0x6E07_079F).unwrap()).unwrap(); // ins v31.b[3], v28.b[0]
    assert_eq!(cpu.simd[31], 0x0011_2233_4455_6677_8899_aabb_aadd_eeff);

    cpu.simd[31] = 0x0f0e_0d0c_0b0a_0908_0706_0504_0302_0100;
    execute(&mut cpu, &mut bus, decode(0x6E20_0BFF).unwrap()).unwrap(); // rev32 v31.16b, v31.16b
    assert_eq!(cpu.simd[31], 0x0c0d_0e0f_0809_0a0b_0405_0607_0001_0203);

    cpu.simd[30] = 0x1122_3344_5566_7788;
    execute(&mut cpu, &mut bus, decode(0x0EA0_0BDE).unwrap()).unwrap(); // rev64 v30.2s, v30.2s
    assert_eq!(cpu.simd[30], 0x5566_7788_1122_3344);

    cpu.simd[30] = 0x1122_3344_5566_7788_99aa_bbcc_ddee_ff00;
    execute(&mut cpu, &mut bus, decode(0x4EA0_0BDE).unwrap()).unwrap(); // rev64 v30.4s, v30.4s
    assert_eq!(cpu.simd[30], 0x5566_7788_1122_3344_ddee_ff00_99aa_bbcc);

    cpu.simd[30] = 0x8000_0001_0001_0001;
    execute(&mut cpu, &mut bus, decode(0x0F2D_57C2).unwrap()).unwrap(); // shl v2.2s, v30.2s, #13
    assert_eq!(cpu.simd[2], 0x0000_2000_2000_2000);

    cpu.simd[4] = 0x0000_0040_0000_007f_0000_0002_0000_0001;
    cpu.simd[6] = 0x0000_0000_5555_5555_aaaa_aaaa_ffff_ffff;
    execute(&mut cpu, &mut bus, decode(0x6F39_5486).unwrap()).unwrap(); // sli v6.4s, v4.4s, #25
    assert_eq!(cpu.simd[6], 0x8000_0000_ff55_5555_04aa_aaaa_03ff_ffff);

    cpu.simd[30] = vector_bytes(0x02);
    cpu.simd[31] = 0x8080_8080_8080_8080_8080_8080_8080_8080;
    execute(&mut cpu, &mut bus, decode(0x6F0F_47DF).unwrap()).unwrap(); // sri v31.16b, v30.16b, #1
    let mut expected_sri = 0u128;
    for lane in 0..16u128 {
        expected_sri |= (0x80 | ((lane + 2) >> 1)) << (lane * 8);
    }
    assert_eq!(cpu.simd[31], expected_sri);

    cpu.simd[0] = 0x7f80_ff01_0010_f0f8;
    execute(&mut cpu, &mut bus, decode(0x0F08_0401).unwrap()).unwrap(); // sshr v1.8b, v0.8b, #8
    assert_eq!(cpu.simd[1], 0x00ff_ff00_0000_ffff);

    cpu.simd[31] = 0x8000_0000_ffff_ff00_7fff_ff00_0000_0100;
    execute(&mut cpu, &mut bus, decode(0x4F38_07FC).unwrap()).unwrap(); // sshr v28.4s, v31.4s, #8
    assert_eq!(cpu.simd[28], 0xff80_0000_ffff_ffff_007f_ffff_0000_0001);

    cpu.simd[11] = 0x8000_0000_0000_0000_7fff_ffff_ffff_ffff;
    execute(&mut cpu, &mut bus, decode(0x4F41_0561).unwrap()).unwrap(); // sshr v1.2d, v11.2d, #63
    assert_eq!(cpu.simd[1], 0xffff_ffff_ffff_ffff_0000_0000_0000_0000);

    cpu.simd[31] = 0xffff_ffff_ffff_0000;
    execute(&mut cpu, &mut bus, decode(0x5F70_07FD).unwrap()).unwrap(); // sshr d29, d31, #16
    assert_eq!(cpu.simd[29], u64::MAX as u128);

    let mut shrn_source = 0u128;
    for lane in 0..8u128 {
        shrn_source |= ((lane + 1) * 0x40) << (lane * 16);
    }
    cpu.simd[31] = shrn_source;
    execute(&mut cpu, &mut bus, decode(0x0F0A_87FF).unwrap()).unwrap(); // shrn v31.8b, v31.8h, #6
    assert_eq!(cpu.simd[31], 0x0807_0605_0403_0201);

    cpu.simd[27] = 0x8877_6655_4433_2211_0123_4567_89ab_cdef;
    execute(&mut cpu, &mut bus, decode(0x4E08_077D).unwrap()).unwrap(); // dup v29.2d, v27.d[0]
    assert_eq!(cpu.simd[29], 0x0123_4567_89ab_cdef_0123_4567_89ab_cdef);

    cpu.simd[29] = 0x8000_0000_0000_0000_0000_0000_0000_0010;
    cpu.simd[25] = 0xffff_ffff_ffff_fffc_0000_0000_0000_0004;
    execute(&mut cpu, &mut bus, decode(0x6EF9_47BD).unwrap()).unwrap(); // ushl v29.2d, v29.2d, v25.2d
    assert_eq!(cpu.simd[29], 0x0800_0000_0000_0000_0000_0000_0000_0100);

    cpu.simd[30] = 1;
    cpu.simd[31] = 0x8000_0000_0000_0002;
    execute(&mut cpu, &mut bus, decode(0x7EFF_47DF).unwrap()).unwrap(); // ushl d31, d30, d31
    assert_eq!(cpu.simd[31], 4);

    cpu.simd[30] = 0x80;
    cpu.simd[31] = 0xff;
    execute(&mut cpu, &mut bus, decode(0x7EFF_47DF).unwrap()).unwrap(); // ushl d31, d30, d31
    assert_eq!(cpu.simd[31], 0x40);
}
