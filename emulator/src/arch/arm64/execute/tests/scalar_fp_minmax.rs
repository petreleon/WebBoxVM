use super::simd_helpers::*;
use super::*;

#[test]
fn scalar_fp_minmax_forms_propagate_nan_and_order_signed_zero() {
    let (mut cpu, mut bus) = setup();

    cpu.simd[1] = f32::NAN.to_bits() as u128;
    cpu.simd[2] = 4.0f32.to_bits() as u128;
    execute(&mut cpu, &mut bus, decode(0x1E22_4820).unwrap()).unwrap();
    assert!(f32_lane(&cpu, 0).is_nan());

    cpu.simd[10] = 3.0f64.to_bits() as u128;
    cpu.simd[11] = f64::NAN.to_bits() as u128;
    execute(&mut cpu, &mut bus, decode(0x1E6B_5949).unwrap()).unwrap();
    assert!(f64_lane(&cpu, 9).is_nan());

    cpu.simd[7] = (-0.0f64).to_bits() as u128;
    cpu.simd[8] = 0.0f64.to_bits() as u128;
    execute(&mut cpu, &mut bus, decode(0x1E68_48E6).unwrap()).unwrap();
    assert_eq!(cpu.simd[6] as u64, 0.0f64.to_bits());

    cpu.simd[4] = (-0.0f32).to_bits() as u128;
    cpu.simd[5] = 0.0f32.to_bits() as u128;
    execute(&mut cpu, &mut bus, decode(0x1E25_5883).unwrap()).unwrap();
    assert_eq!(cpu.simd[3] as u32, (-0.0f32).to_bits());
}
