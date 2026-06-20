use super::simd_helpers::*;
use super::*;

#[test]
fn simd_userland_arithmetic_ops() {
    let (mut cpu, mut bus) = setup();

    cpu.regs.set_w(4, 0x3f80_0000);
    execute(&mut cpu, &mut bus, decode(0x1E27_009F).unwrap()).unwrap(); // fmov s31, w4
    assert_eq!(cpu.simd[31], 0x3f80_0000);

    cpu.simd[31] = 0xffff_ffff_8000_0001;
    cpu.regs.set_x(1, u64::MAX);
    execute(&mut cpu, &mut bus, decode(0x1E26_03E1).unwrap()).unwrap(); // fmov w1, s31
    assert_eq!(cpu.regs.x(1), 0x8000_0001);

    cpu.simd[0] = 0x1111_1111_2222_2222;
    cpu.regs.set_x(3, 0xaaaa_bbbb_cccc_dddd);
    execute(&mut cpu, &mut bus, decode(0x9EAF_0060).unwrap()).unwrap(); // fmov v0.d[1], x3
    assert_eq!(cpu.simd[0], 0xaaaa_bbbb_cccc_dddd_1111_1111_2222_2222);

    cpu.simd[31] = ((10u128) << 64) | 2;
    cpu.simd[30] = ((u64::MAX as u128) << 64) | 3;
    execute(&mut cpu, &mut bus, decode(0x4EFE_87FF).unwrap()).unwrap(); // add v31.2d, v31.2d, v30.2d
    assert_eq!(cpu.simd[31], ((9u128) << 64) | 5);

    cpu.simd[31] = ((10u128) << 64) | 5;
    cpu.simd[29] = ((3u128) << 64) | 2;
    execute(&mut cpu, &mut bus, decode(0x6EFD_87FF).unwrap()).unwrap(); // sub v31.2d, v31.2d, v29.2d
    assert_eq!(cpu.simd[31], ((7u128) << 64) | 3);

    cpu.simd[0] = vector_bytes(1);
    cpu.simd[2] = vector_bytes(2);
    execute(&mut cpu, &mut bus, decode(0x4E22_9C03).unwrap()).unwrap(); // mul v3.16b, v0.16b, v2.16b
    let mut expected_bytes = 0u128;
    for lane in 0..16u128 {
        expected_bytes |= (((lane + 1) * (lane + 2)) & 0xff) << (lane * 8);
    }
    assert_eq!(cpu.simd[3], expected_bytes);

    cpu.simd[5] = 0x1234_ffff_8000_0002;
    cpu.simd[6] = 0x0004_0002_0002_0003;
    execute(&mut cpu, &mut bus, decode(0x0E66_9CA4).unwrap()).unwrap(); // mul v4.4h, v5.4h, v6.4h
    assert_eq!(cpu.simd[4], 0x48d0_fffe_0000_0006);

    cpu.simd[29] = 0xffff_fffe_0000_0003;
    cpu.simd[30] = 0x0000_0003_0000_0005;
    execute(&mut cpu, &mut bus, decode(0x0EBE_9FBD).unwrap()).unwrap(); // mul v29.2s, v29.2s, v30.2s
    assert_eq!(cpu.simd[29], 0xffff_fffa_0000_000f);

    cpu.simd[30] = 0x0000_0001_ffff_ffff_0000_0010_0000_0002;
    cpu.simd[31] = 0x0000_0004_0000_0003_0000_0002_ffff_ffff;
    execute(&mut cpu, &mut bus, decode(0x4EBF_97FE).unwrap()).unwrap(); // mla v30.4s, v31.4s, v31.4s
    assert_eq!(cpu.simd[30], 0x0000_0011_0000_0008_0000_0014_0000_0003);

    cpu.simd[5] = u16x8([3, 0, 0x1000, 0x8000, 0xffff, 5, 0x2222, 1]);
    cpu.simd[2] = u16x8([0xffff, 2, 0x0100, 2, 3, 0xffff, 0x8000, 0x1234]);
    cpu.simd[23] = u16x8([2, 3, 0x0010, 0x8000, 2, 0xffff, 2, 0x0010]);
    execute(&mut cpu, &mut bus, decode(0x6E77_9445).unwrap()).unwrap(); // mls v5.8h, v2.8h, v23.8h
    assert_eq!(
        cpu.simd[5],
        u16x8([5, 0xfffa, 0, 0x8000, 0xfff9, 4, 0x2222, 0xdcc1])
    );

    cpu.simd[31] = 0x8000_0000_0000_0003_0000_0002_ffff_ffff;
    execute(&mut cpu, &mut bus, decode(0x6EA0_BBFF).unwrap()).unwrap(); // neg v31.4s, v31.4s
    assert_eq!(cpu.simd[31], 0x8000_0000_ffff_fffd_ffff_fffe_0000_0001);

    cpu.simd[31] = 0x8000_0000_0000_0003;
    execute(&mut cpu, &mut bus, decode(0x7EE0_BBFF).unwrap()).unwrap(); // neg d31, d31
    assert_eq!(cpu.simd[31], 0x7fff_ffff_ffff_fffd);

    cpu.simd[31] = ((-3.0f64).to_bits() as u128) << 64 | 2.0f64.to_bits() as u128;
    execute(&mut cpu, &mut bus, decode(0x6EE0_FBFF).unwrap()).unwrap(); // fneg v31.2d, v31.2d
    assert_eq!(
        cpu.simd[31],
        (3.0f64.to_bits() as u128) << 64 | (-2.0f64).to_bits() as u128
    );

    cpu.simd[30] = 0x0000_0005_8000_0000_0000_0000_0000_00ff;
    cpu.simd[22] = 0x0000_0008_8000_0000_0000_007b_0000_0f0f;
    execute(&mut cpu, &mut bus, decode(0x4EB6_8FDF).unwrap()).unwrap(); // cmtst v31.4s, v30.4s, v22.4s
    assert_eq!(cpu.simd[31], 0x0000_0000_ffff_ffff_0000_0000_ffff_ffff);
}

#[test]
fn simd_addhn_uses_decoded_rm_register() {
    let (mut cpu, mut bus) = setup();

    cpu.simd[7] = 0xf000_1111_abcd_1234_ffff_8000_0100_00ff;
    cpu.simd[8] = 0x1000_eeee_1111_0101_0001_8000_0200_0001;

    execute(&mut cpu, &mut bus, decode(0x0E28_40E6).unwrap()).unwrap();

    assert_eq!(cpu.simd[6], 0x00ff_bc13_0000_0301);
}

#[test]
fn simd_subhn_keeps_high_half_of_wrapping_difference() {
    let (mut cpu, mut bus) = setup();

    cpu.simd[0] = 0x0001_0000_8000_ffff_1234_0000_0200_0100;
    cpu.simd[30] = 0x0002_0001_0001_0000_1200_0001_0100_0300;
    execute(&mut cpu, &mut bus, decode(0x0E7E_6002).unwrap()).unwrap(); // subhn v2.4h, v0.4s, v30.4s
    assert_eq!(cpu.simd[2], 0xfffe_7fff_0033_00ff);

    cpu.simd[2] = 0x0000_0000_0000_0000_0000_0002_0000_0000;
    cpu.simd[5] = 0x0000_0001_0000_0000_0000_0000_0000_0001;
    execute(&mut cpu, &mut bus, decode(0x0EA5_6042).unwrap()).unwrap(); // subhn v2.2s, v2.2d, v5.2d
    assert_eq!(cpu.simd[2], 0xffff_ffff_0000_0001);
}
