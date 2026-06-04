use super::simd_helpers::*;
use super::*;

#[test]
fn simd_reduce_across_forms_write_signed_and_unsigned_scalar_results() {
    let (mut cpu, mut bus) = setup();

    cpu.simd[1] = 0x7f80_fe02_0100_0000_0000_0000_0000_0000;
    execute(&mut cpu, &mut bus, decode(0x4E30_A820).unwrap()).unwrap();
    assert_eq!(cpu.simd[0], 0x7f);

    cpu.simd[1] = i32x4([5, -7, i32::MIN, 9]);
    execute(&mut cpu, &mut bus, decode(0x4EB1_A820).unwrap()).unwrap();
    assert_eq!(cpu.simd[0], i32::MIN as u32 as u128);

    cpu.simd[1] = 0x0009_0002_ffff_0001_0008_0003_0004_0005;
    execute(&mut cpu, &mut bus, decode(0x6E71_A820).unwrap()).unwrap();
    assert_eq!(cpu.simd[0], 1);
}

#[test]
fn simd_fp_reduce_across_s_forms_write_scalar_results() {
    let (mut cpu, mut bus) = setup();

    cpu.simd[1] = f32x4([-1.5, 7.0, 3.0, -9.0]);
    execute(&mut cpu, &mut bus, decode(0x6E30_F820).unwrap()).unwrap();
    assert_eq!(f32_lane(&cpu, 0), 7.0);
    assert_eq!(cpu.simd[0] >> 32, 0);

    cpu.simd[3] = f32x4([-0.0, 5.0, -4.0, 0.0]);
    execute(&mut cpu, &mut bus, decode(0x6EB0_F862).unwrap()).unwrap();
    assert_eq!(f32_lane(&cpu, 2), -4.0);

    cpu.simd[5] = f32x4([f32::NAN, 6.5, -1.0, 3.25]);
    execute(&mut cpu, &mut bus, decode(0x6E30_C8A4).unwrap()).unwrap();
    assert_eq!(f32_lane(&cpu, 4), 6.5);

    cpu.simd[7] = f32x4([f32::NAN, 6.5, -2.0, f32::NAN]);
    execute(&mut cpu, &mut bus, decode(0x6EB0_C8E6).unwrap()).unwrap();
    assert_eq!(f32_lane(&cpu, 6), -2.0);

    cpu.simd[1] = f32x4([1.0, f32::NAN, 2.0, 3.0]);
    execute(&mut cpu, &mut bus, decode(0x6E30_F820).unwrap()).unwrap();
    assert!(f32_lane(&cpu, 0).is_nan());
}
