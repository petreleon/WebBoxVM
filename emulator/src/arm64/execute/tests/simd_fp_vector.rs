use super::simd_helpers::*;
use super::*;

#[test]
fn simd_fp_vector_arithmetic_ops() {
    let (mut cpu, mut bus) = setup();

    cpu.simd[0] = f32x4([1.0, -2.0, 3.5, 4.0]);
    execute(&mut cpu, &mut bus, decode(0x4E20_D400).unwrap()).unwrap(); // fadd v0.4s, v0.4s, v0.4s
    assert_eq!(cpu.simd[0], f32x4([2.0, -4.0, 7.0, 8.0]));

    cpu.simd[2] = f32x4([10.0, 20.0, -3.0, 5.0]);
    cpu.simd[3] = f32x4([1.0, 2.0, 4.0, 7.0]);
    execute(&mut cpu, &mut bus, decode(0x4EA3_D441).unwrap()).unwrap(); // fsub v1.4s, v2.4s, v3.4s
    assert_eq!(cpu.simd[1], f32x4([9.0, 18.0, -7.0, -2.0]));

    cpu.simd[5] = f32x4([2.0, -3.0, 0.5, 8.0]);
    cpu.simd[6] = f32x4([4.0, 5.0, -2.0, 0.25]);
    execute(&mut cpu, &mut bus, decode(0x6E26_DCA4).unwrap()).unwrap(); // fmul v4.4s, v5.4s, v6.4s
    assert_eq!(cpu.simd[4], f32x4([8.0, -15.0, -1.0, 2.0]));

    cpu.simd[8] = f64x2([9.0, -8.0]);
    cpu.simd[9] = f64x2([3.0, 2.0]);
    execute(&mut cpu, &mut bus, decode(0x6E69_FD07).unwrap()).unwrap(); // fdiv v7.2d, v8.2d, v9.2d
    assert_eq!(cpu.simd[7], f64x2([3.0, -4.0]));

    cpu.simd[1] = f32x4([-1.0, 5.5, 8.0, -4.0]);
    cpu.simd[2] = f32x4([2.5, 1.5, -2.0, -9.0]);
    execute(&mut cpu, &mut bus, decode(0x6EA2_D420).unwrap()).unwrap(); // fabd v0.4s, v1.4s, v2.4s
    assert_eq!(cpu.simd[0], f32x4([3.5, 4.0, 10.0, 5.0]));

    cpu.simd[1] = 9.5f64.to_bits() as u128;
    cpu.simd[2] = (-2.25f64).to_bits() as u128;
    execute(&mut cpu, &mut bus, decode(0x7EE2_D420).unwrap()).unwrap(); // fabd d0, d1, d2
    assert_eq!(f64_lane(&cpu, 0), 11.75);

    cpu.simd[0] = i64x2([3, -4]);
    execute(&mut cpu, &mut bus, decode(0x4E61_D800).unwrap()).unwrap(); // scvtf v0.2d, v0.2d
    assert_eq!(cpu.simd[0], f64x2([3.0, -4.0]));

    cpu.simd[1] = i32x4([2, -3, 4, -5]);
    execute(&mut cpu, &mut bus, decode(0x4E21_D821).unwrap()).unwrap(); // scvtf v1.4s, v1.4s
    assert_eq!(cpu.simd[1], f32x4([2.0, -3.0, 4.0, -5.0]));

    cpu.simd[2] = u32x4([2, u32::MAX, 4, 5]);
    execute(&mut cpu, &mut bus, decode(0x6E21_D842).unwrap()).unwrap(); // ucvtf v2.4s, v2.4s
    assert_eq!(cpu.simd[2], f32x4([2.0, 4_294_967_296.0, 4.0, 5.0]));

    cpu.simd[3] = u64x2([3, u64::MAX]);
    execute(&mut cpu, &mut bus, decode(0x6E61_D863).unwrap()).unwrap(); // ucvtf v3.2d, v3.2d
    assert_eq!(cpu.simd[3], f64x2([3.0, 18_446_744_073_709_551_616.0]));

    cpu.simd[30] = u64::MAX as u128;
    execute(&mut cpu, &mut bus, decode(0x7E61_DBDE).unwrap()).unwrap(); // ucvtf d30, d30
    assert_eq!(f64_lane(&cpu, 30), 18_446_744_073_709_551_616.0);

    cpu.simd[31] = (-7i64 as u64) as u128;
    execute(&mut cpu, &mut bus, decode(0x5E61_DBFF).unwrap()).unwrap(); // scvtf d31, d31
    assert_eq!(f64_lane(&cpu, 31), -7.0);

    cpu.simd[0] = f64x2([3.9, -2.1]);
    execute(&mut cpu, &mut bus, decode(0x4EE1_B800).unwrap()).unwrap(); // fcvtzs v0.2d, v0.2d
    assert_eq!(cpu.simd[0], i64x2([3, -2]));

    cpu.simd[1] = f32x4([1.9, -1.9, 7.0, -8.5]);
    execute(&mut cpu, &mut bus, decode(0x4EA1_B821).unwrap()).unwrap(); // fcvtzs v1.4s, v1.4s
    assert_eq!(cpu.simd[1], i32x4([1, -1, 7, -8]));

    cpu.simd[31] = (-2.75f64).to_bits() as u128;
    execute(&mut cpu, &mut bus, decode(0x5EE1_BBFF).unwrap()).unwrap(); // fcvtzs d31, d31
    assert_eq!(cpu.simd[31], (-2i64 as u64) as u128);
}
