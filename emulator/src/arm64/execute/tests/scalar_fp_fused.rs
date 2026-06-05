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

    cpu.simd[1] = 2.0f64.to_bits() as u128;
    cpu.simd[2] = 3.0f64.to_bits() as u128;
    cpu.simd[3] = 4.0f64.to_bits() as u128;
    execute(&mut cpu, &mut bus, decode(0x1F62_0C24).unwrap()).unwrap(); // fnmadd d4, d1, d2, d3
    assert_eq!(f64_lane(&cpu, 4), -10.0);

    cpu.simd[5] = 2.0f32.to_bits() as u128;
    cpu.simd[6] = 3.0f32.to_bits() as u128;
    cpu.simd[7] = 4.0f32.to_bits() as u128;
    execute(&mut cpu, &mut bus, decode(0x1F26_1CA8).unwrap()).unwrap(); // fnmadd s8, s5, s6, s7
    assert_eq!(f32_lane(&cpu, 8), -10.0);

    cpu.simd[31] = 2.0f64.to_bits() as u128;
    cpu.simd[22] = 3.0f64.to_bits() as u128;
    cpu.simd[25] = 4.0f64.to_bits() as u128;
    execute(&mut cpu, &mut bus, decode(0x1F76_E7F9).unwrap()).unwrap(); // fnmsub d25, d31, d22, d25
    assert_eq!(f64_lane(&cpu, 25), 2.0);

    cpu.simd[16] = 0x4000;
    cpu.simd[17] = 0x4200;
    cpu.simd[18] = 0x4400;
    execute(&mut cpu, &mut bus, decode(0x1FD1_4A0F).unwrap()).unwrap(); // fmadd h15, h16, h17, h18
    assert_eq!(cpu.simd[15] as u16, 0x4900);

    cpu.simd[20] = 0x4000;
    cpu.simd[21] = 0x4200;
    cpu.simd[22] = 0x4900;
    execute(&mut cpu, &mut bus, decode(0x1FD5_DA93).unwrap()).unwrap(); // fmsub h19, h20, h21, h22
    assert_eq!(cpu.simd[19] as u16, 0x4400);

    cpu.simd[24] = 0x4000;
    cpu.simd[25] = 0x4200;
    cpu.simd[26] = 0x4400;
    execute(&mut cpu, &mut bus, decode(0x1FF9_6B17).unwrap()).unwrap(); // fnmadd h23, h24, h25, h26
    assert_eq!(cpu.simd[23] as u16, 0xc900);

    cpu.simd[28] = 0x4000;
    cpu.simd[29] = 0x4200;
    cpu.simd[30] = 0x4400;
    execute(&mut cpu, &mut bus, decode(0x1FFD_FB9B).unwrap()).unwrap(); // fnmsub h27, h28, h29, h30
    assert_eq!(cpu.simd[27] as u16, 0x4000);
}
