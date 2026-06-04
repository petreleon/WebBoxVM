use super::simd_helpers::*;
use super::*;

#[test]
fn simd_reduce_across_forms_write_signed_and_unsigned_scalar_results() {
    let (mut cpu, mut bus) = setup();

    cpu.simd[1] = 0x7f80_fe02_0100_0000_0000_0000_0000_0000;
    execute(&mut cpu, &mut bus, decode(0x4E30_A820).unwrap()).unwrap();
    assert_eq!(cpu.simd[0], 0x7f);

    cpu.simd[1] = i32x4([5, -7, i32::MIN, 9]);
    execute(&mut cpu, &mut bus, decode(0x4EB1_A820).unwrap()).unwrap();
    assert_eq!(cpu.simd[0], i32::MIN as u32 as u128);

    cpu.simd[1] = 0x0009_0002_ffff_0001_0008_0003_0004_0005;
    execute(&mut cpu, &mut bus, decode(0x6E71_A820).unwrap()).unwrap();
    assert_eq!(cpu.simd[0], 1);
}
