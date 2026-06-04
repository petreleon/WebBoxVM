use super::simd_helpers::*;
use super::*;

#[test]
fn scalar_fp_fused_multiply_add_ops() {
    let (mut cpu, mut bus) = setup();

    cpu.simd[28] = 2.0f64.to_bits() as u128;
    cpu.simd[27] = 3.0f64.to_bits() as u128;
    cpu.simd[30] = 4.0f64.to_bits() as u128;
    execute(&mut cpu, &mut bus, decode(0x1F5B_7B9E).unwrap()).unwrap(); // fmadd d30, d28, d27, d30
    assert_eq!(f64_lane(&cpu, 30), 10.0);

    cpu.simd[29] = 2.0f64.to_bits() as u128;
    cpu.simd[31] = 3.0f64.to_bits() as u128;
    cpu.simd[30] = 10.0f64.to_bits() as u128;
    execute(&mut cpu, &mut bus, decode(0x1F5F_FBBE).unwrap()).unwrap(); // fmsub d30, d29, d31, d30
    assert_eq!(f64_lane(&cpu, 30), 4.0);

    cpu.simd[31] = 2.0f64.to_bits() as u128;
    cpu.simd[22] = 3.0f64.to_bits() as u128;
    cpu.simd[25] = 4.0f64.to_bits() as u128;
    execute(&mut cpu, &mut bus, decode(0x1F76_E7F9).unwrap()).unwrap(); // fnmsub d25, d31, d22, d25
    assert_eq!(f64_lane(&cpu, 25), 2.0);
}
