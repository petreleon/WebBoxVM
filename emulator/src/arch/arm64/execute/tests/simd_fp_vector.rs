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

    cpu.simd[26] = f32x4([10.0, 20.0, 30.0, 40.0]);
    cpu.simd[27] = f32x4([1.0, 2.0, 3.0, 4.0]);
    cpu.simd[30] = f32x4([0.5, 1.5, 2.5, 3.5]);
    execute(&mut cpu, &mut bus, decode(0x4E3E_CF7A).unwrap()).unwrap(); // fmla v26.4s, v27.4s, v30.4s
    assert_eq!(cpu.simd[26], f32x4([10.5, 23.0, 37.5, 54.0]));

    cpu.simd[28] = f32x4([10.0, 20.0, 30.0, 40.0]);
    cpu.simd[24] = f32x4([1.0, 2.0, 3.0, 4.0]);
    execute(&mut cpu, &mut bus, decode(0x4EBC_CF1C).unwrap()).unwrap(); // fmls v28.4s, v24.4s, v28.4s
    assert_eq!(cpu.simd[28], f32x4([0.0, -20.0, -60.0, -120.0]));

    cpu.simd[15] = f32x4([10.0, 20.0, 30.0, 40.0]);
    cpu.simd[31] = f32x4([1.0, 2.0, 3.0, 4.0]);
    cpu.simd[25] = f32x4([0.5, 1.5, 2.5, 3.5]);
    execute(&mut cpu, &mut bus, decode(0x4FB9_1BEF).unwrap()).unwrap(); // fmla v15.4s, v31.4s, v25.s[3]
    assert_eq!(cpu.simd[15], f32x4([13.5, 27.0, 40.5, 54.0]));

    cpu.simd[28] = f32x4([10.0, 20.0, 30.0, 40.0]);
    cpu.simd[24] = f32x4([1.0, 2.0, 3.0, 4.0]);
    execute(&mut cpu, &mut bus, decode(0x4F99_53FC).unwrap()).unwrap(); // fmls v28.4s, v31.4s, v25.s[0]
    assert_eq!(cpu.simd[28], f32x4([9.5, 19.0, 28.5, 38.0]));

    execute(&mut cpu, &mut bus, decode(0x4F99_9BFE).unwrap()).unwrap(); // fmul v30.4s, v31.4s, v25.s[2]
    assert_eq!(cpu.simd[30], f32x4([2.5, 5.0, 7.5, 10.0]));

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

    cpu.simd[1] = f32x4([-3.0, 2.0, -1.0, 4.0]);
    cpu.simd[29] = f32x4([2.0, -2.0, 1.5, -5.0]);
    execute(&mut cpu, &mut bus, decode(0x6EBD_EC21).unwrap()).unwrap(); // facgt v1.4s, v1.4s, v29.4s
    assert_eq!(cpu.simd[1], u32x4([u32::MAX, 0, 0, 0]));

    cpu.simd[1] = f32x4([-3.0, 2.0, -1.0, 4.0]);
    cpu.simd[0] = f32x4([3.0, -2.5, 0.5, -4.0]);
    execute(&mut cpu, &mut bus, decode(0x6E20_EC21).unwrap()).unwrap(); // facge v1.4s, v1.4s, v0.4s
    assert_eq!(cpu.simd[1], u32x4([u32::MAX, 0, u32::MAX, u32::MAX]));

    cpu.simd[0] = f32x4([-1.0, 0.0, -0.0, 2.0]);
    execute(&mut cpu, &mut bus, decode(0x4EA0_E803).unwrap()).unwrap(); // fcmlt v3.4s, v0.4s, #0
    assert_eq!(cpu.simd[3], u32x4([u32::MAX, 0, 0, 0]));

    cpu.simd[30] = f64x2([-2.5, 3.0]);
    execute(&mut cpu, &mut bus, decode(0x4EE0_EBC6).unwrap()).unwrap(); // fcmlt v6.2d, v30.2d, #0
    assert_eq!(cpu.simd[6], u64x2([u64::MAX, 0]));

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

    cpu.simd[0] = f32x4([1.4, 1.5, -1.5, -2.6]);
    execute(&mut cpu, &mut bus, decode(0x4E21_C802).unwrap()).unwrap(); // fcvtas v2.4s, v0.4s
    assert_eq!(cpu.simd[2], i32x4([1, 2, -2, -3]));

    cpu.simd[29] = f64x2([2.5, -2.5]);
    execute(&mut cpu, &mut bus, decode(0x4E61_CBBB).unwrap()).unwrap(); // fcvtas v27.2d, v29.2d
    assert_eq!(cpu.simd[27], i64x2([3, -3]));

    cpu.simd[29] = f32x4([1.5, -2.25, 3.0, -4.5]);
    execute(&mut cpu, &mut bus, decode(0x0E61_7BA4).unwrap()).unwrap(); // fcvtl v4.2d, v29.2s
    assert_eq!(cpu.simd[4], f64x2([1.5, -2.25]));
    execute(&mut cpu, &mut bus, decode(0x4E61_7BA7).unwrap()).unwrap(); // fcvtl2 v7.2d, v29.4s
    assert_eq!(cpu.simd[7], f64x2([3.0, -4.5]));

    cpu.simd[0] = u16x8([0x3c00, 0xc000, 0x4000, 0, 0x4400, 0x4500, 0, 0]);
    execute(&mut cpu, &mut bus, decode(0x0E21_7800).unwrap()).unwrap(); // fcvtl v0.4s, v0.4h
    assert_eq!(cpu.simd[0], f32x4([1.0, -2.0, 2.0, 0.0]));

    cpu.simd[2] = f64x2([1.5, -2.75]);
    execute(&mut cpu, &mut bus, decode(0x0E61_6842).unwrap()).unwrap(); // fcvtn v2.2s, v2.2d
    assert_eq!(cpu.simd[2], f32x4([1.5, -2.75, 0.0, 0.0]));

    cpu.simd[2] = f32x4([9.0, 8.0, 99.0, 99.0]);
    cpu.simd[30] = f64x2([3.5, -4.25]);
    execute(&mut cpu, &mut bus, decode(0x4E61_6BC2).unwrap()).unwrap(); // fcvtn2 v2.4s, v30.2d
    assert_eq!(cpu.simd[2], f32x4([9.0, 8.0, 3.5, -4.25]));

    cpu.simd[31] = (-2.75f64).to_bits() as u128;
    execute(&mut cpu, &mut bus, decode(0x5EE1_BBFF).unwrap()).unwrap(); // fcvtzs d31, d31
    assert_eq!(cpu.simd[31], (-2i64 as u64) as u128);
}
