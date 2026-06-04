use super::simd_helpers::*;
use super::*;

#[test]
fn simd_scalar_bitwise_compares_update_single_d_lane() {
    let (mut cpu, mut bus) = setup();

    cpu.simd[1] = u64x2([0x0f0f, 0xffff]);
    cpu.simd[0] = u64x2([0x0f00, 0]);
    execute(&mut cpu, &mut bus, decode(0x5EE0_8C20).unwrap()).unwrap();
    assert_eq!(cpu.simd[0], u64::MAX as u128);

    cpu.simd[1] = u64x2([0x1234, 0]);
    cpu.simd[0] = u64x2([0x5678, u64::MAX]);
    execute(&mut cpu, &mut bus, decode(0x7EE0_8C20).unwrap()).unwrap();
    assert_eq!(cpu.simd[0], 0);
}
