use super::*;

#[test]
fn simd_word_immediates_and_cmeq_zero() {
    let (mut cpu, mut bus) = setup();

    execute(&mut cpu, &mut bus, decode(0x0F00_043F).unwrap()).unwrap(); // movi v31.2s, #1
    assert_eq!(cpu.simd[31], 0x0000_0001_0000_0001);

    execute(&mut cpu, &mut bus, decode(0x0F00_848F).unwrap()).unwrap(); // movi v15.4h, #4
    assert_eq!(cpu.simd[15], 0x0004_0004_0004_0004);

    execute(&mut cpu, &mut bus, decode(0x0F04_A41F).unwrap()).unwrap(); // movi v31.4h, #0x80, LSL #8
    assert_eq!(cpu.simd[31], 0x8000_8000_8000_8000);

    execute(&mut cpu, &mut bus, decode(0x0F03_E7FF).unwrap()).unwrap(); // movi v31.8b, #0x7f
    assert_eq!(cpu.simd[31], 0x7f7f_7f7f_7f7f_7f7f);

    execute(&mut cpu, &mut bus, decode(0x2F00_051E).unwrap()).unwrap(); // mvni v30.2s, #8
    assert_eq!(cpu.simd[30], 0xffff_fff7_ffff_fff7);

    execute(&mut cpu, &mut bus, decode(0x2F04_A480).unwrap()).unwrap(); // mvni v0.4h, #0x84, LSL #8
    assert_eq!(cpu.simd[0], 0x7bff_7bff_7bff_7bff);

    execute(&mut cpu, &mut bus, decode(0x2F03_D7FE).unwrap()).unwrap(); // mvni v30.2s, #0x7f, MSL #16
    assert_eq!(cpu.simd[30], 0xff80_0000_ff80_0000);

    execute(&mut cpu, &mut bus, decode(0x2F07_E61F).unwrap()).unwrap(); // movi d31, #0xffffffff00000000
    assert_eq!(cpu.simd[31], 0xffff_ffff_0000_0000);

    cpu.simd[0] = 0x0001_0000_00ff_0000;
    execute(&mut cpu, &mut bus, decode(0x0E20_9800).unwrap()).unwrap(); // cmeq v0.8b, v0.8b, #0
    assert_eq!(cpu.simd[0], 0xff00_ffff_ff00_ffff);

    cpu.simd[31] = 0;
    execute(&mut cpu, &mut bus, decode(0x5EE0_9BFF).unwrap()).unwrap(); // cmeq d31, d31, #0
    assert_eq!(cpu.simd[31], u64::MAX as u128);

    cpu.simd[31] = 1;
    execute(&mut cpu, &mut bus, decode(0x5EE0_9BFF).unwrap()).unwrap(); // cmeq d31, d31, #0
    assert_eq!(cpu.simd[31], 0);

    cpu.simd[31] = u64::MAX as u128;
    execute(&mut cpu, &mut bus, decode(0x7EE0_8BFF).unwrap()).unwrap(); // cmge d31, d31, #0
    assert_eq!(cpu.simd[31], 0);

    cpu.simd[31] = 0x7fff_ffff_ffff_ffff;
    execute(&mut cpu, &mut bus, decode(0x7EE0_8BFF).unwrap()).unwrap(); // cmge d31, d31, #0
    assert_eq!(cpu.simd[31], u64::MAX as u128);

    cpu.simd[30] = 0x8000_0000_0000_0001_7fff_ffff_ffff_ffff;
    execute(&mut cpu, &mut bus, decode(0x6EE0_8BDE).unwrap()).unwrap(); // cmge v30.2d, v30.2d, #0
    assert_eq!(cpu.simd[30], 0x0000_0000_0000_0000_ffff_ffff_ffff_ffff);
}
