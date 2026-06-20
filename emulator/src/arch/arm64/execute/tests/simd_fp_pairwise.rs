use super::simd_helpers::*;
use super::*;

#[test]
fn simd_fp_pairwise_add_forms_group_source_lanes() {
    let (mut cpu, mut bus) = setup();

    cpu.simd[1] = f32x4([1.0, 2.5, 99.0, 100.0]);
    cpu.simd[2] = f32x4([10.0, -4.0, 77.0, 88.0]);
    execute(&mut cpu, &mut bus, decode(0x2E22_D420).unwrap()).unwrap();
    assert_eq!(cpu.simd[0], f32x4([3.5, 6.0, 0.0, 0.0]));

    cpu.simd[4] = f32x4([1.0, 2.0, 3.0, 4.0]);
    cpu.simd[5] = f32x4([10.0, 20.0, 30.0, 40.0]);
    execute(&mut cpu, &mut bus, decode(0x6E25_D483).unwrap()).unwrap();
    assert_eq!(cpu.simd[3], f32x4([3.0, 7.0, 30.0, 70.0]));

    cpu.simd[7] = f64x2([1.25, 2.5]);
    cpu.simd[8] = f64x2([10.0, -4.0]);
    execute(&mut cpu, &mut bus, decode(0x6E68_D4E6).unwrap()).unwrap();
    assert_eq!(cpu.simd[6], f64x2([3.75, 6.0]));
}

#[test]
fn simd_fp_pairwise_scalar_add_forms_write_scalar_result() {
    let (mut cpu, mut bus) = setup();

    cpu.simd[10] = f32x4([7.5, -1.25, 99.0, 100.0]);
    execute(&mut cpu, &mut bus, decode(0x7E30_D949).unwrap()).unwrap();
    assert_eq!(f32_lane(&cpu, 9), 6.25);
    assert_eq!(cpu.simd[9] >> 32, 0);

    cpu.simd[12] = f64x2([3.0, -0.5]);
    execute(&mut cpu, &mut bus, decode(0x7E70_D98B).unwrap()).unwrap();
    assert_eq!(f64_lane(&cpu, 11), 2.5);
    assert_eq!(cpu.simd[11] >> 64, 0);
}
