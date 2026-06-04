use super::simd_helpers::*;
use super::*;

#[test]
fn scalar_fp_conversion_ops() {
    let (mut cpu, mut bus) = setup();

    cpu.simd[0] = 2.25f64.to_bits() as u128;
    execute(&mut cpu, &mut bus, decode(0x1E62_401F).unwrap()).unwrap(); // fcvt s31, d0
    assert_eq!(f32_lane(&cpu, 31), 2.25);

    cpu.simd[0] = 1.5f32.to_bits() as u128;
    execute(&mut cpu, &mut bus, decode(0x1E22_C000).unwrap()).unwrap(); // fcvt d0, s0
    assert_eq!(f64_lane(&cpu, 0), 1.5);

    cpu.simd[0] = 0x3e00;
    execute(&mut cpu, &mut bus, decode(0x1EE2_4015).unwrap()).unwrap(); // fcvt s21, h0
    assert_eq!(f32_lane(&cpu, 21), 1.5);

    cpu.simd[0] = 0xc000;
    execute(&mut cpu, &mut bus, decode(0x1EE2_C015).unwrap()).unwrap(); // fcvt d21, h0
    assert_eq!(f64_lane(&cpu, 21), -2.0);

    cpu.simd[30] = 1.000_488_281_25f32.to_bits() as u128;
    execute(&mut cpu, &mut bus, decode(0x1E23_C3DE).unwrap()).unwrap(); // fcvt h30, s30
    assert_eq!(cpu.simd[30], 0x3c00);

    cpu.simd[28] = 65_504.0f64.to_bits() as u128;
    execute(&mut cpu, &mut bus, decode(0x1E63_C39C).unwrap()).unwrap(); // fcvt h28, d28
    assert_eq!(cpu.simd[28], 0x7bff);

    cpu.simd[28] = f64::INFINITY.to_bits() as u128;
    execute(&mut cpu, &mut bus, decode(0x1E63_C39C).unwrap()).unwrap(); // fcvt h28, d28
    assert_eq!(cpu.simd[28], 0x7c00);

    cpu.simd[30] = 0xffff_ffff_4000_0000;
    execute(&mut cpu, &mut bus, decode(0x1E20_43DD).unwrap()).unwrap(); // fmov s29, s30
    assert_eq!(cpu.simd[29], 0x4000_0000);
    assert_eq!(f32_lane(&cpu, 29), 2.0);

    cpu.simd[0] = (-2.25f64).to_bits() as u128;
    execute(&mut cpu, &mut bus, decode(0x1E65_4000).unwrap()).unwrap(); // frintm d0, d0
    assert_eq!(f64_lane(&cpu, 0), -3.0);

    cpu.simd[0] = 2.5f64.to_bits() as u128;
    execute(&mut cpu, &mut bus, decode(0x1E64_4000).unwrap()).unwrap(); // frintn d0, d0
    assert_eq!(f64_lane(&cpu, 0), 2.0);

    cpu.simd[31] = 2.5f64.to_bits() as u128;
    execute(&mut cpu, &mut bus, decode(0x1E66_43FF).unwrap()).unwrap(); // frinta d31, d31
    assert_eq!(f64_lane(&cpu, 31), 3.0);

    cpu.simd[31] = (-2.5f64).to_bits() as u128;
    execute(&mut cpu, &mut bus, decode(0x1E66_43FF).unwrap()).unwrap(); // frinta d31, d31
    assert_eq!(f64_lane(&cpu, 31), -3.0);

    cpu.sys.fpcr = 0;
    cpu.simd[0] = 2.5f64.to_bits() as u128;
    execute(&mut cpu, &mut bus, decode(0x1E67_4000).unwrap()).unwrap(); // frintx d0, d0
    assert_eq!(f64_lane(&cpu, 0), 2.0);

    cpu.simd[31] = (-2.9f64).to_bits() as u128;
    execute(&mut cpu, &mut bus, decode(0x1E65_C3FF).unwrap()).unwrap(); // frintz d31, d31
    assert_eq!(f64_lane(&cpu, 31), -2.0);

    cpu.regs.set_w(1, 7);
    execute(&mut cpu, &mut bus, decode(0x1E63_003F).unwrap()).unwrap(); // ucvtf d31, w1
    assert_eq!(f64_lane(&cpu, 31), 7.0);

    cpu.regs.set_x(0, 1u64 << 40);
    execute(&mut cpu, &mut bus, decode(0x9E63_001F).unwrap()).unwrap(); // ucvtf d31, x0
    assert_eq!(f64_lane(&cpu, 31), (1u64 << 40) as f64);

    cpu.regs.set_w(0, 6);
    execute(&mut cpu, &mut bus, decode(0x1E03_FC00).unwrap()).unwrap(); // ucvtf s0, w0, #1
    assert_eq!(f32_lane(&cpu, 0), 3.0);

    cpu.simd[31] = (-3.9f64).to_bits() as u128;
    execute(&mut cpu, &mut bus, decode(0x1E78_03E0).unwrap()).unwrap(); // fcvtzs w0, d31
    assert_eq!(cpu.regs.w(0), (-3i32) as u32);

    cpu.simd[31] = 3.9f64.to_bits() as u128;
    execute(&mut cpu, &mut bus, decode(0x1E79_03E0).unwrap()).unwrap(); // fcvtzu w0, d31
    assert_eq!(cpu.regs.w(0), 3);

    cpu.simd[0] = 5.9f64.to_bits() as u128;
    execute(&mut cpu, &mut bus, decode(0x9E79_0000).unwrap()).unwrap(); // fcvtzu x0, d0
    assert_eq!(cpu.regs.x(0), 5);

    cpu.simd[31] = 1.75f64.to_bits() as u128;
    execute(&mut cpu, &mut bus, decode(0x1E58_FBE0).unwrap()).unwrap(); // fcvtzs w0, d31, #2
    assert_eq!(cpu.regs.w(0), 7);

    cpu.simd[29] = 3.75f32.to_bits() as u128;
    execute(&mut cpu, &mut bus, decode(0x1E19_F3A1).unwrap()).unwrap(); // fcvtzu w1, s29, #4
    assert_eq!(cpu.regs.w(1), 60);

    cpu.simd[31] = (-2.1f64).to_bits() as u128;
    execute(&mut cpu, &mut bus, decode(0x1E70_03E1).unwrap()).unwrap(); // fcvtms w1, d31
    assert_eq!(cpu.regs.w(1), (-3i32) as u32);

    cpu.simd[0] = 2.5f64.to_bits() as u128;
    execute(&mut cpu, &mut bus, decode(0x9E60_0002).unwrap()).unwrap(); // fcvtns x2, d0
    assert_eq!(cpu.regs.x(2), 2);

    cpu.simd[0] = 3.5f64.to_bits() as u128;
    execute(&mut cpu, &mut bus, decode(0x9E60_0002).unwrap()).unwrap(); // fcvtns x2, d0
    assert_eq!(cpu.regs.x(2), 4);

    cpu.simd[29] = (-2.5f64).to_bits() as u128;
    execute(&mut cpu, &mut bus, decode(0x9E64_03A3).unwrap()).unwrap(); // fcvtas x3, d29
    assert_eq!(cpu.regs.x(3) as i64, -3);

    cpu.simd[0] = 2.5f32.to_bits() as u128;
    execute(&mut cpu, &mut bus, decode(0x9E24_0000).unwrap()).unwrap(); // fcvtas x0, s0
    assert_eq!(cpu.regs.x(0), 3);

    cpu.simd[0] = 5.9f64.to_bits() as u128;
    execute(&mut cpu, &mut bus, decode(0x7EE1_B800).unwrap()).unwrap(); // fcvtzu d0, d0
    assert_eq!(cpu.simd[0], 5);

    cpu.simd[0] = 3.9f32.to_bits() as u128;
    execute(&mut cpu, &mut bus, decode(0x7EA1_B800).unwrap()).unwrap(); // fcvtzu s0, s0
    assert_eq!(cpu.simd[0], 3);
}
