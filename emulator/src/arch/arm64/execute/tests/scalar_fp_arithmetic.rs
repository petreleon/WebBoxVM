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

#[test]
fn scalar_fp_half_arithmetic_unary_rounding_ops() {
    let (mut cpu, mut bus) = setup();
    const H_ONE: u16 = 0x3c00;
    const H_TWO: u16 = 0x4000;
    const H_THREE: u16 = 0x4200;
    const H_FOUR: u16 = 0x4400;
    const H_FIVE: u16 = 0x4500;
    const H_SIX: u16 = 0x4600;
    const H_EIGHT: u16 = 0x4800;
    const H_TWO_HALF: u16 = 0x4100;
    const H_NEG_TWO: u16 = 0xc000;
    const H_NEG_THREE: u16 = 0xc200;
    const H_NEG_EIGHT: u16 = 0xc800;
    const H_NEG_TWO_HALF: u16 = 0xc100;

    execute(&mut cpu, &mut bus, decode(0x1EEE_100A).unwrap()).unwrap(); // fmov h10, #1
    assert_eq!(cpu.simd[10] as u16, H_ONE);

    cpu.simd[1] = H_ONE as u128;
    cpu.simd[2] = H_TWO as u128;
    execute(&mut cpu, &mut bus, decode(0x1EE2_2820).unwrap()).unwrap(); // fadd h0, h1, h2
    assert_eq!(cpu.simd[0] as u16, H_THREE);

    cpu.simd[4] = H_FIVE as u128;
    cpu.simd[5] = H_TWO as u128;
    execute(&mut cpu, &mut bus, decode(0x1EE5_3883).unwrap()).unwrap(); // fsub h3, h4, h5
    assert_eq!(cpu.simd[3] as u16, H_THREE);

    cpu.simd[7] = H_TWO as u128;
    cpu.simd[8] = H_FOUR as u128;
    execute(&mut cpu, &mut bus, decode(0x1EE8_08E6).unwrap()).unwrap(); // fmul h6, h7, h8
    assert_eq!(cpu.simd[6] as u16, H_EIGHT);

    cpu.simd[10] = H_TWO as u128;
    cpu.simd[11] = H_FOUR as u128;
    execute(&mut cpu, &mut bus, decode(0x1EEB_8949).unwrap()).unwrap(); // fnmul h9, h10, h11
    assert_eq!(cpu.simd[9] as u16, H_NEG_EIGHT);

    cpu.simd[13] = H_SIX as u128;
    cpu.simd[14] = H_TWO as u128;
    execute(&mut cpu, &mut bus, decode(0x1EEE_19AC).unwrap()).unwrap(); // fdiv h12, h13, h14
    assert_eq!(cpu.simd[12] as u16, H_THREE);

    cpu.simd[1] = H_THREE as u128;
    execute(&mut cpu, &mut bus, decode(0x1EE1_4020).unwrap()).unwrap(); // fneg h0, h1
    assert_eq!(cpu.simd[0] as u16, H_NEG_THREE);

    cpu.simd[3] = H_NEG_THREE as u128;
    execute(&mut cpu, &mut bus, decode(0x1EE0_C062).unwrap()).unwrap(); // fabs h2, h3
    assert_eq!(cpu.simd[2] as u16, H_THREE);

    cpu.simd[5] = H_FOUR as u128;
    execute(&mut cpu, &mut bus, decode(0x1EE1_C0A4).unwrap()).unwrap(); // fsqrt h4, h5
    assert_eq!(cpu.simd[4] as u16, H_TWO);

    cpu.simd[7] = H_TWO_HALF as u128;
    execute(&mut cpu, &mut bus, decode(0x1EE4_40E6).unwrap()).unwrap(); // frintn h6, h7
    assert_eq!(cpu.simd[6] as u16, H_TWO);

    cpu.simd[9] = H_NEG_TWO_HALF as u128;
    execute(&mut cpu, &mut bus, decode(0x1EE5_C128).unwrap()).unwrap(); // frintz h8, h9
    assert_eq!(cpu.simd[8] as u16, H_NEG_TWO);
}
