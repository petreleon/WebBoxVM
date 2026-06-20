use super::*;

#[test]
fn simd_raddhn_rounds_and_raddhn2_preserves_lower() {
    let (mut cpu, mut bus) = setup();

    cpu.simd[6] = 0xaaaa_aaaa_aaaa_aaaa_1122_3344_5566_7788;
    cpu.simd[7] = 0xf000_1111_abcd_1234_ffff_8000_0100_00ff;
    cpu.simd[8] = 0x1000_eeee_1111_0101_0001_8000_0200_0001;

    execute(&mut cpu, &mut bus, decode(0x2E28_40E6).unwrap()).unwrap();
    assert_eq!(cpu.simd[6], 0x0000_bd13_0000_0301);

    cpu.simd[6] = 0xaaaa_aaaa_aaaa_aaaa_1122_3344_5566_7788;
    execute(&mut cpu, &mut bus, decode(0x6E28_40E6).unwrap()).unwrap();
    assert_eq!(cpu.simd[6], 0x0000_bd13_0000_0301_1122_3344_5566_7788);
}

#[test]
fn simd_rsubhn_rounds_negative_results_and_preserves_lower() {
    let (mut cpu, mut bus) = setup();

    cpu.simd[2] = 0xbbbb_bbbb_bbbb_bbbb_0123_4567_89ab_cdef;
    cpu.simd[0] = 0x0001_0000_8000_ffff_1234_0000_0200_0100;
    cpu.simd[30] = 0x0002_0001_0001_0000_1200_0001_0100_0300;

    execute(&mut cpu, &mut bus, decode(0x2E7E_6002).unwrap()).unwrap();
    assert_eq!(cpu.simd[2], 0xffff_8000_0034_0100);

    cpu.simd[2] = 0xbbbb_bbbb_bbbb_bbbb_0123_4567_89ab_cdef;
    execute(&mut cpu, &mut bus, decode(0x6E7E_6002).unwrap()).unwrap();
    assert_eq!(cpu.simd[2], 0xffff_8000_0034_0100_0123_4567_89ab_cdef);
}

#[test]
fn simd_rshrn_rounds_and_rshrn2_preserves_lower() {
    let (mut cpu, mut bus) = setup();

    let mut source = 0u128;
    for lane in 0..8u128 {
        source |= (lane * 0x40 + 0x20) << (lane * 16);
    }
    cpu.simd[2] = 0xbbbb_bbbb_bbbb_bbbb_8877_6655_4433_2211;
    cpu.simd[31] = source;

    execute(&mut cpu, &mut bus, decode(0x0F0A_8FE2).unwrap()).unwrap();
    assert_eq!(cpu.simd[2], 0x0807_0605_0403_0201);

    cpu.simd[2] = 0xbbbb_bbbb_bbbb_bbbb_8877_6655_4433_2211;
    execute(&mut cpu, &mut bus, decode(0x4F0A_8FE2).unwrap()).unwrap();
    assert_eq!(cpu.simd[2], 0x0807_0605_0403_0201_8877_6655_4433_2211);
}
