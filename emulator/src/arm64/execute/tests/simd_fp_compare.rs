use super::simd_helpers::*;
use super::*;

#[test]
fn simd_fp_vector_compare_forms() {
    let (mut cpu, mut bus) = setup();

    cpu.simd[1] = f32x4([3.0, -0.0, f32::NAN, 4.0]);
    cpu.simd[2] = f32x4([3.0, 0.0, f32::NAN, 5.0]);
    execute(&mut cpu, &mut bus, decode(0x0E22_E420).unwrap()).unwrap();
    assert_eq!(cpu.simd[0], u32x4([u32::MAX, u32::MAX, 0, 0]));

    cpu.simd[7] = f64x2([1.0, f64::NAN]);
    cpu.simd[8] = f64x2([1.0, f64::NAN]);
    execute(&mut cpu, &mut bus, decode(0x4E68_E4E6).unwrap()).unwrap();
    assert_eq!(cpu.simd[6], u64x2([u64::MAX, 0]));

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

#[test]
fn simd_fp_ge_gt_zero_compare_forms_use_literal_zero() {
    let (mut cpu, mut bus) = setup();

    cpu.simd[1] = f32x4([0.0, -1.0, 99.0, 100.0]);
    execute(&mut cpu, &mut bus, decode(0x2EA0_C820).unwrap()).unwrap();
    assert_eq!(cpu.simd[0], u32x4([u32::MAX, 0, 0, 0]));

    cpu.simd[3] = f32x4([1.0, 0.0, -1.0, f32::NAN]);
    execute(&mut cpu, &mut bus, decode(0x4EA0_C862).unwrap()).unwrap();
    assert_eq!(cpu.simd[2], u32x4([u32::MAX, 0, 0, 0]));

    cpu.simd[5] = f64x2([-0.0, -1.0]);
    execute(&mut cpu, &mut bus, decode(0x6EE0_C8A4).unwrap()).unwrap();
    assert_eq!(cpu.simd[4], u64x2([u64::MAX, 0]));

    cpu.simd[7] = f64x2([2.0, 0.0]);
    execute(&mut cpu, &mut bus, decode(0x4EE0_C8E6).unwrap()).unwrap();
    assert_eq!(cpu.simd[6], u64x2([u64::MAX, 0]));
}

#[test]
fn simd_fp_scalar_compare_forms_write_scalar_masks() {
    let (mut cpu, mut bus) = setup();

    cpu.simd[10] = f32x4([-0.0, 99.0, 99.0, 99.0]);
    cpu.simd[11] = f32x4([0.0, 99.0, 99.0, 99.0]);
    execute(&mut cpu, &mut bus, decode(0x5E2B_E549).unwrap()).unwrap();
    assert_eq!(cpu.simd[9], u32::MAX as u128);

    cpu.simd[1] = f32x4([3.0, 99.0, 99.0, 99.0]);
    cpu.simd[2] = f32x4([2.0, 99.0, 99.0, 99.0]);
    execute(&mut cpu, &mut bus, decode(0x7E22_E420).unwrap()).unwrap();
    assert_eq!(cpu.simd[0], u32::MAX as u128);

    cpu.simd[4] = f32x4([5.0, 99.0, 99.0, 99.0]);
    cpu.simd[5] = f32x4([5.0, 99.0, 99.0, 99.0]);
    execute(&mut cpu, &mut bus, decode(0x7EA5_E483).unwrap()).unwrap();
    assert_eq!(cpu.simd[3], 0);

    cpu.simd[7] = f32x4([-5.0, 99.0, 99.0, 99.0]);
    cpu.simd[8] = f32x4([4.0, 99.0, 99.0, 99.0]);
    execute(&mut cpu, &mut bus, decode(0x7E28_ECE6).unwrap()).unwrap();
    assert_eq!(cpu.simd[6], u32::MAX as u128);

    cpu.simd[28] = f64x2([-3.0, 99.0]);
    cpu.simd[29] = f64x2([2.0, 99.0]);
    execute(&mut cpu, &mut bus, decode(0x7EFD_EF9B).unwrap()).unwrap();
    assert_eq!(cpu.simd[27], u64::MAX as u128);

    cpu.simd[13] = f64x2([f64::NAN, 99.0]);
    cpu.simd[14] = f64x2([f64::NAN, 99.0]);
    execute(&mut cpu, &mut bus, decode(0x5E6E_E5AC).unwrap()).unwrap();
    assert_eq!(cpu.simd[12], 0);
}

#[test]
fn simd_fp_scalar_zero_compare_forms_write_scalar_masks() {
    let (mut cpu, mut bus) = setup();

    cpu.simd[1] = f32x4([-0.0, 99.0, 99.0, 99.0]);
    execute(&mut cpu, &mut bus, decode(0x7EA0_C820).unwrap()).unwrap();
    assert_eq!(cpu.simd[0], u32::MAX as u128);

    cpu.simd[3] = f32x4([0.0, 99.0, 99.0, 99.0]);
    execute(&mut cpu, &mut bus, decode(0x5EA0_C862).unwrap()).unwrap();
    assert_eq!(cpu.simd[2], 0);

    cpu.simd[13] = f32x4([-0.0, 99.0, 99.0, 99.0]);
    execute(&mut cpu, &mut bus, decode(0x5EA0_D9AC).unwrap()).unwrap();
    assert_eq!(cpu.simd[12], u32::MAX as u128);

    cpu.simd[15] = f32x4([0.0, 99.0, 99.0, 99.0]);
    execute(&mut cpu, &mut bus, decode(0x7EA0_D9EE).unwrap()).unwrap();
    assert_eq!(cpu.simd[14], u32::MAX as u128);

    cpu.simd[3] = f64x2([-1.0, 99.0]);
    execute(&mut cpu, &mut bus, decode(0x5EE0_E862).unwrap()).unwrap();
    assert_eq!(cpu.simd[2], u64::MAX as u128);
}
