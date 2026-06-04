use super::simd_helpers::*;
use super::*;

#[test]
fn simd_fp_vector_compare_forms() {
    let (mut cpu, mut bus) = setup();

    cpu.simd[31] = f32x4([3.0, 1.0, f32::NAN, -2.0]);
    cpu.simd[0] = f32x4([2.0, 1.0, 0.0, -3.0]);
    execute(&mut cpu, &mut bus, decode(0x6E20_E7FE).unwrap()).unwrap();
    assert_eq!(cpu.simd[30], u32x4([u32::MAX, u32::MAX, 0, u32::MAX]));

    cpu.simd[27] = f32x4([5.0, 5.0, f32::NAN, -1.0]);
    cpu.simd[28] = f32x4([4.0, 5.0, 0.0, -2.0]);
    execute(&mut cpu, &mut bus, decode(0x6EBC_E77C).unwrap()).unwrap();
    assert_eq!(cpu.simd[28], u32x4([u32::MAX, 0, 0, u32::MAX]));

    cpu.simd[7] = f64x2([3.0, -4.0]);
    cpu.simd[27] = f64x2([2.0, -5.0]);
    execute(&mut cpu, &mut bus, decode(0x6EFB_E4FB).unwrap()).unwrap();
    assert_eq!(cpu.simd[27], u64x2([u64::MAX, u64::MAX]));
}

#[test]
fn simd_fp_zero_compare_forms_use_literal_zero() {
    let (mut cpu, mut bus) = setup();

    cpu.simd[0] = f64x2([42.0, -1.0]);
    cpu.simd[29] = f64x2([0.0, -0.0]);
    execute(&mut cpu, &mut bus, decode(0x4EE0_DBA0).unwrap()).unwrap();
    assert_eq!(cpu.simd[0], u64x2([u64::MAX, u64::MAX]));

    cpu.simd[0] = f32x4([7.0, 8.0, 9.0, 10.0]);
    cpu.simd[1] = f32x4([-1.0, 0.0, 1.0, f32::NAN]);
    execute(&mut cpu, &mut bus, decode(0x6EA0_D83C).unwrap()).unwrap();
    assert_eq!(cpu.simd[28], u32x4([u32::MAX, u32::MAX, 0, 0]));
}
