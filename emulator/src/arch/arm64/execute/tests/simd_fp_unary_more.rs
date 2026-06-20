use super::simd_helpers::*;
use super::*;

#[test]
fn simd_fp_abs_and_round_vectors() {
    let (mut cpu, mut bus) = setup();

    cpu.simd[0] = f32x4([-1.0, 2.5, -0.0, -7.25]);
    execute(&mut cpu, &mut bus, decode(0x4EA0_F81F).unwrap()).unwrap(); // fabs v31.4s, v0.4s
    assert_eq!(cpu.simd[31], f32x4([1.0, 2.5, 0.0, 7.25]));

    cpu.simd[0] = f64x2([-3.5, 4.25]);
    execute(&mut cpu, &mut bus, decode(0x4EE0_F81D).unwrap()).unwrap(); // fabs v29.2d, v0.2d
    assert_eq!(cpu.simd[29], f64x2([3.5, 4.25]));

    cpu.simd[0] = f32x4([1.5, 2.5, -1.5, -2.5]);
    execute(&mut cpu, &mut bus, decode(0x4E21_881F).unwrap()).unwrap(); // frintn v31.4s, v0.4s
    assert_eq!(cpu.simd[31], f32x4([2.0, 2.0, -2.0, -2.0]));

    cpu.simd[0] = f32x4([3.5, -4.5, 99.0, 100.0]);
    execute(&mut cpu, &mut bus, decode(0x0E21_8801).unwrap()).unwrap(); // frintn v1.2s, v0.2s
    assert_eq!(cpu.simd[1], f32x4([4.0, -4.0, 0.0, 0.0]));

    cpu.simd[1] = 0x4000_bc00_3e00_3c00; // [1.0h, 1.5h, -1.0h, 2.0h]
    execute(&mut cpu, &mut bus, decode(0x0E79_8822).unwrap()).unwrap(); // frintn v2.4h, v1.4h
    assert_eq!(cpu.simd[2], 0x4000_bc00_4000_3c00);

    cpu.simd[29] = f64x2([3.5, 4.5]);
    execute(&mut cpu, &mut bus, decode(0x4E61_8BA6).unwrap()).unwrap(); // frintn v6.2d, v29.2d
    assert_eq!(cpu.simd[6], f64x2([4.0, 4.0]));

    cpu.simd[28] = f32x4([1.4, -1.5, 2.5, -2.6]);
    execute(&mut cpu, &mut bus, decode(0x6E21_8B9C).unwrap()).unwrap(); // frinta v28.4s, v28.4s
    assert_eq!(cpu.simd[28], f32x4([1.0, -2.0, 3.0, -3.0]));

    cpu.simd[28] = f64x2([3.5, -4.5]);
    execute(&mut cpu, &mut bus, decode(0x6E61_8B9C).unwrap()).unwrap(); // frinta v28.2d, v28.2d
    assert_eq!(cpu.simd[28], f64x2([4.0, -5.0]));

    cpu.simd[30] = f32x4([4.0, 9.0, 16.0, 25.0]);
    execute(&mut cpu, &mut bus, decode(0x6EA1_FBC5).unwrap()).unwrap(); // fsqrt v5.4s, v30.4s
    assert_eq!(cpu.simd[5], f32x4([2.0, 3.0, 4.0, 5.0]));

    cpu.simd[31] = f64x2([36.0, 49.0]);
    execute(&mut cpu, &mut bus, decode(0x6EE1_FBE4).unwrap()).unwrap(); // fsqrt v4.2d, v31.2d
    assert_eq!(cpu.simd[4], f64x2([6.0, 7.0]));
}
