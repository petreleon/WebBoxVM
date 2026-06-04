use super::simd_helpers::*;
use super::*;

#[test]
fn simd_usra_accumulates_unsigned_shifted_lanes() {
    let (mut cpu, mut bus) = setup();

    cpu.simd[28] = u32x4([10, u32::MAX - 1, 100, 0]);
    cpu.simd[5] = u32x4([8, 4, u32::MAX, 2]);
    execute(&mut cpu, &mut bus, decode(0x6F3F_14BC).unwrap()).unwrap();
    assert_eq!(cpu.simd[28], u32x4([14, 0, 0x8000_0063, 1]));

    cpu.simd[0] = vector_bytes(1);
    cpu.simd[29] = vector_bytes(0x80);
    execute(&mut cpu, &mut bus, decode(0x6F09_17A0).unwrap()).unwrap();
    assert_eq!(cpu.simd[0], vector_bytes(2));

    cpu.simd[28] = 0xffff_ffff_0000_0000;
    cpu.simd[5] = u32x4([2, 8, 16, 32]);
    execute(&mut cpu, &mut bus, decode(0x2F3F_14BC).unwrap()).unwrap();
    assert_eq!(cpu.simd[28], u32x4([1, 3, 0, 0]));
}
