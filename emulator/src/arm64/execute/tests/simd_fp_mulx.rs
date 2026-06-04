use super::simd_helpers::*;
use super::*;

#[test]
fn simd_fp_mulx_vector_forms_apply_extended_multiply_rule() {
    let (mut cpu, mut bus) = setup();

    cpu.simd[7] = f32x4([0.0, -3.0, 99.0, 100.0]);
    cpu.simd[8] = f32x4([f32::INFINITY, 2.0, 77.0, 88.0]);
    execute(&mut cpu, &mut bus, decode(0x0E28_DCE6).unwrap()).unwrap();
    assert_eq!(cpu.simd[6], f32x4([2.0, -6.0, 0.0, 0.0]));

    cpu.simd[10] = f32x4([1.0, -0.0, 3.0, -4.0]);
    cpu.simd[11] = f32x4([2.0, f32::INFINITY, -5.0, 0.5]);
    execute(&mut cpu, &mut bus, decode(0x4E2B_DD49).unwrap()).unwrap();
    assert_eq!(cpu.simd[9], f32x4([2.0, -2.0, -15.0, -2.0]));

    cpu.simd[13] = f64x2([0.0, 1.5]);
    cpu.simd[14] = f64x2([f64::NEG_INFINITY, -2.0]);
    execute(&mut cpu, &mut bus, decode(0x4E6E_DDAC).unwrap()).unwrap();
    assert_eq!(cpu.simd[12], f64x2([-2.0, -3.0]));
}

#[test]
fn simd_fp_mulx_scalar_forms_write_scalar_result() {
    let (mut cpu, mut bus) = setup();

    cpu.simd[1] = 0.0f32.to_bits() as u128;
    cpu.simd[2] = f32::INFINITY.to_bits() as u128;
    execute(&mut cpu, &mut bus, decode(0x5E22_DC20).unwrap()).unwrap();
    assert_eq!(f32_lane(&cpu, 0), 2.0);
    assert_eq!(cpu.simd[0] >> 32, 0);

    cpu.simd[4] = (-0.0f64).to_bits() as u128;
    cpu.simd[5] = f64::INFINITY.to_bits() as u128;
    execute(&mut cpu, &mut bus, decode(0x5E65_DC83).unwrap()).unwrap();
    assert_eq!(f64_lane(&cpu, 3), -2.0);
    assert_eq!(cpu.simd[3] >> 64, 0);
}

#[test]
fn simd_fp_mulx_by_element_forms_select_source_lane() {
    let (mut cpu, mut bus) = setup();

    cpu.simd[1] = f32x4([0.0, -3.0, 99.0, 100.0]);
    cpu.simd[2] = f32x4([f32::INFINITY, 2.0, 77.0, 88.0]);
    execute(&mut cpu, &mut bus, decode(0x2F82_9020).unwrap()).unwrap();
    assert_eq!(cpu.simd[0], f32x4([2.0, f32::NEG_INFINITY, 0.0, 0.0]));

    cpu.simd[4] = f32x4([1.0, -0.0, 3.0, -4.0]);
    cpu.simd[5] = f32x4([9.0, 8.0, 7.0, 0.5]);
    execute(&mut cpu, &mut bus, decode(0x6FA5_9883).unwrap()).unwrap();
    assert_eq!(cpu.simd[3], f32x4([0.5, -0.0, 1.5, -2.0]));

    cpu.simd[7] = f64x2([0.0, 1.5]);
    cpu.simd[8] = f64x2([9.0, f64::NEG_INFINITY]);
    execute(&mut cpu, &mut bus, decode(0x6FC8_98E6).unwrap()).unwrap();
    assert_eq!(cpu.simd[6], f64x2([-2.0, f64::NEG_INFINITY]));
}

#[test]
fn simd_fp_mulx_scalar_by_element_forms_write_scalar_result() {
    let (mut cpu, mut bus) = setup();

    cpu.simd[10] = (-0.0f32).to_bits() as u128;
    cpu.simd[11] = f32x4([1.0, 2.0, f32::INFINITY, 4.0]);
    execute(&mut cpu, &mut bus, decode(0x7F8B_9949).unwrap()).unwrap();
    assert_eq!(f32_lane(&cpu, 9), -2.0);
    assert_eq!(cpu.simd[9] >> 32, 0);

    cpu.simd[16] = 0.0f64.to_bits() as u128;
    cpu.simd[17] = f64x2([1.0, f64::NEG_INFINITY]);
    execute(&mut cpu, &mut bus, decode(0x7FD1_9A0F).unwrap()).unwrap();
    assert_eq!(f64_lane(&cpu, 15), -2.0);
    assert_eq!(cpu.simd[15] >> 64, 0);
}
