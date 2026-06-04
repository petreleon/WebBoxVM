use super::simd_helpers::*;
use super::*;

#[test]
fn scalar_fp_arithmetic_and_rounding_ops() {
    let (mut cpu, mut bus) = setup();

    execute(&mut cpu, &mut bus, decode(0x1E6E_1000).unwrap()).unwrap(); // fmov d0, #1
    assert_eq!(f64_lane(&cpu, 0), 1.0);

    execute(&mut cpu, &mut bus, decode(0x1E62_900F).unwrap()).unwrap(); // fmov d15, #5
    assert_eq!(f64_lane(&cpu, 15), 5.0);

    cpu.regs.set_w(0, 8);
    execute(&mut cpu, &mut bus, decode(0x1E42_F800).unwrap()).unwrap(); // scvtf d0, w0, #2
    assert_eq!(f64_lane(&cpu, 0), 2.0);

    cpu.regs.set_w(20, (-2i32) as u32);
    execute(&mut cpu, &mut bus, decode(0x1E22_0280).unwrap()).unwrap(); // scvtf s0, w20
    assert_eq!(f32_lane(&cpu, 0), -2.0);

    cpu.simd[0] = 1.5f64.to_bits() as u128;
    cpu.simd[31] = 2.0f64.to_bits() as u128;
    execute(&mut cpu, &mut bus, decode(0x1E7F_0800).unwrap()).unwrap(); // fmul d0, d0, d31
    assert_eq!(f64_lane(&cpu, 0), 3.0);

    cpu.simd[0] = 2.0f64.to_bits() as u128;
    cpu.simd[31] = 4.0f64.to_bits() as u128;
    execute(&mut cpu, &mut bus, decode(0x1E7F_8800).unwrap()).unwrap(); // fnmul d0, d0, d31
    assert_eq!(f64_lane(&cpu, 0), -8.0);

    cpu.simd[0] = 1.5f32.to_bits() as u128;
    cpu.simd[1] = (-2.0f32).to_bits() as u128;
    execute(&mut cpu, &mut bus, decode(0x1E21_8800).unwrap()).unwrap(); // fnmul s0, s0, s1
    assert_eq!(f32_lane(&cpu, 0), 3.0);

    cpu.simd[25] = 0.25f64.to_bits() as u128;
    cpu.simd[31] = 2.0f64.to_bits() as u128;
    execute(&mut cpu, &mut bus, decode(0x1E79_2BFF).unwrap()).unwrap(); // fadd d31, d31, d25
    assert_eq!(f64_lane(&cpu, 31), 2.25);

    cpu.simd[28] = 4.0f64.to_bits() as u128;
    cpu.simd[27] = 1.5f64.to_bits() as u128;
    execute(&mut cpu, &mut bus, decode(0x1E7B_3B9A).unwrap()).unwrap(); // fsub d26, d28, d27
    assert_eq!(f64_lane(&cpu, 26), 2.5);

    cpu.simd[31] = 6.0f64.to_bits() as u128;
    cpu.simd[0] = 2.0f64.to_bits() as u128;
    execute(&mut cpu, &mut bus, decode(0x1E60_1BE0).unwrap()).unwrap(); // fdiv d0, d31, d0
    assert_eq!(f64_lane(&cpu, 0), 3.0);

    cpu.simd[0] = (-3.0f64).to_bits() as u128;
    cpu.simd[1] = 5.0f64.to_bits() as u128;
    execute(&mut cpu, &mut bus, decode(0x1E61_6800).unwrap()).unwrap(); // fmaxnm d0, d0, d1
    assert_eq!(f64_lane(&cpu, 0), 5.0);

    cpu.simd[0] = (-0.0f64).to_bits() as u128;
    cpu.simd[1] = 0.0f64.to_bits() as u128;
    execute(&mut cpu, &mut bus, decode(0x1E61_7800).unwrap()).unwrap(); // fminnm d0, d0, d1
    assert_eq!(cpu.simd[0] as u64, (-0.0f64).to_bits());

    cpu.simd[0] = f32::NAN.to_bits() as u128;
    cpu.simd[1] = 4.0f32.to_bits() as u128;
    execute(&mut cpu, &mut bus, decode(0x1E21_6800).unwrap()).unwrap(); // fmaxnm s0, s0, s1
    assert_eq!(f32_lane(&cpu, 0), 4.0);

    cpu.simd[0] = 3.0f64.to_bits() as u128;
    execute(&mut cpu, &mut bus, decode(0x1E61_401F).unwrap()).unwrap(); // fneg d31, d0
    assert_eq!(f64_lane(&cpu, 31), -3.0);

    execute(&mut cpu, &mut bus, decode(0x1E60_C000).unwrap()).unwrap(); // fabs d0, d0
    assert_eq!(f64_lane(&cpu, 0), 3.0);

    cpu.simd[0] = 9.0f64.to_bits() as u128;
    execute(&mut cpu, &mut bus, decode(0x1E61_C000).unwrap()).unwrap(); // fsqrt d0, d0
    assert_eq!(f64_lane(&cpu, 0), 3.0);

    cpu.simd[31] = 2.25f64.to_bits() as u128;
    execute(&mut cpu, &mut bus, decode(0x1E64_C3E0).unwrap()).unwrap(); // frintp d0, d31
    assert_eq!(f64_lane(&cpu, 0), 3.0);

    cpu.sys.fpcr = 0;
    cpu.simd[0] = 2.5f32.to_bits() as u128;
    execute(&mut cpu, &mut bus, decode(0x1E27_C000).unwrap()).unwrap(); // frinti s0, s0
    assert_eq!(f32_lane(&cpu, 0), 2.0);

    cpu.sys.fpcr = 1 << 22;
    cpu.simd[0] = 2.25f64.to_bits() as u128;
    execute(&mut cpu, &mut bus, decode(0x1E67_C000).unwrap()).unwrap(); // frinti d0, d0
    assert_eq!(f64_lane(&cpu, 0), 3.0);
    cpu.sys.fpcr = 0;
}
