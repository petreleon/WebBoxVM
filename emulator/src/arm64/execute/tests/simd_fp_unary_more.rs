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

    cpu.simd[28] = f32x4([1.4, -1.5, 2.5, -2.6]);
    execute(&mut cpu, &mut bus, decode(0x6E21_8B9C).unwrap()).unwrap(); // frinta v28.4s, v28.4s
    assert_eq!(cpu.simd[28], f32x4([1.0, -2.0, 3.0, -3.0]));

    cpu.simd[28] = f64x2([3.5, -4.5]);
    execute(&mut cpu, &mut bus, decode(0x6E61_8B9C).unwrap()).unwrap(); // frinta v28.2d, v28.2d
    assert_eq!(cpu.simd[28], f64x2([4.0, -5.0]));
}
