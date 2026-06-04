use super::simd_helpers::*;
use super::*;

#[test]
fn simd_fp_minmax_vector_forms_update_lanes() {
    let (mut cpu, mut bus) = setup();

    cpu.simd[1] = f32x4([-0.0, -5.0, 7.0, f32::NAN]);
    cpu.simd[2] = f32x4([0.0, 3.0, 2.0, 4.0]);
    execute(&mut cpu, &mut bus, decode(0x4E22_F420).unwrap()).unwrap();
    assert_eq!(u32_lane(cpu.simd[0], 0), 0.0f32.to_bits());
    assert_eq!(f32::from_bits(u32_lane(cpu.simd[0], 1)), 3.0);
    assert_eq!(f32::from_bits(u32_lane(cpu.simd[0], 2)), 7.0);
    assert!(f32::from_bits(u32_lane(cpu.simd[0], 3)).is_nan());

    cpu.simd[4] = f64x2([10.0, -0.0]);
    cpu.simd[5] = f64x2([4.0, 0.0]);
    execute(&mut cpu, &mut bus, decode(0x4EE5_F483).unwrap()).unwrap();
    assert_eq!(f64::from_bits(u64_lane(cpu.simd[3], 0)), 4.0);
    assert_eq!(u64_lane(cpu.simd[3], 1), (-0.0f64).to_bits());

    cpu.simd[7] = f32x4([f32::NAN, 1.0, -0.0, f32::NAN]);
    cpu.simd[8] = f32x4([5.0, f32::NAN, 0.0, f32::NAN]);
    execute(&mut cpu, &mut bus, decode(0x4E28_C4E6).unwrap()).unwrap();
    assert_eq!(f32::from_bits(u32_lane(cpu.simd[6], 0)), 5.0);
    assert_eq!(f32::from_bits(u32_lane(cpu.simd[6], 1)), 1.0);
    assert_eq!(u32_lane(cpu.simd[6], 2), 0.0f32.to_bits());
    assert!(f32::from_bits(u32_lane(cpu.simd[6], 3)).is_nan());

    cpu.simd[10] = f64x2([f64::NAN, -0.0]);
    cpu.simd[11] = f64x2([6.0, 0.0]);
    execute(&mut cpu, &mut bus, decode(0x4EEB_C549).unwrap()).unwrap();
    assert_eq!(f64::from_bits(u64_lane(cpu.simd[9], 0)), 6.0);
    assert_eq!(u64_lane(cpu.simd[9], 1), (-0.0f64).to_bits());
}

#[test]
fn simd_fp_minmax_pairwise_forms_group_source_lanes() {
    let (mut cpu, mut bus) = setup();

    cpu.simd[1] = f32x4([1.0, 7.0, -3.0, 5.0]);
    cpu.simd[2] = f32x4([9.0, -2.0, 6.0, 8.0]);
    execute(&mut cpu, &mut bus, decode(0x6E22_F420).unwrap()).unwrap();
    assert_eq!(cpu.simd[0], f32x4([7.0, 5.0, 9.0, 8.0]));

    cpu.simd[4] = f64x2([10.0, -4.0]);
    cpu.simd[5] = f64x2([3.0, -0.0]);
    execute(&mut cpu, &mut bus, decode(0x6EE5_F483).unwrap()).unwrap();
    assert_eq!(cpu.simd[3], f64x2([-4.0, -0.0]));

    cpu.simd[7] = f32x4([f32::NAN, 1.0, -0.0, 0.0]);
    cpu.simd[8] = f32x4([f32::NAN, 4.0, f32::NAN, f32::NAN]);
    execute(&mut cpu, &mut bus, decode(0x6E28_C4E6).unwrap()).unwrap();
    assert_eq!(f32::from_bits(u32_lane(cpu.simd[6], 0)), 1.0);
    assert_eq!(u32_lane(cpu.simd[6], 1), 0.0f32.to_bits());
    assert_eq!(f32::from_bits(u32_lane(cpu.simd[6], 2)), 4.0);
    assert!(f32::from_bits(u32_lane(cpu.simd[6], 3)).is_nan());

    cpu.simd[10] = f64x2([f64::NAN, 6.0]);
    cpu.simd[11] = f64x2([-0.0, 0.0]);
    execute(&mut cpu, &mut bus, decode(0x6EEB_C549).unwrap()).unwrap();
    assert_eq!(cpu.simd[9], f64x2([6.0, -0.0]));
}
