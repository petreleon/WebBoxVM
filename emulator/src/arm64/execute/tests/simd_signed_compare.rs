use super::simd_helpers::*;
use super::*;

#[test]
fn simd_signed_register_compares_use_signed_lane_ordering() {
    let (mut cpu, mut bus) = setup();

    cpu.simd[25] = i32x4([-1, 5, 7, i32::MIN]);
    cpu.simd[29] = i32x4([0, 5, -6, i32::MAX]);
    execute(&mut cpu, &mut bus, decode(0x4EBD_3F39).unwrap()).unwrap();
    assert_eq!(cpu.simd[25], u32x4([0, u32::MAX, u32::MAX, 0]));

    cpu.simd[25] = i32x4([-1, 5, 7, i32::MIN]);
    execute(&mut cpu, &mut bus, decode(0x4EBD_3739).unwrap()).unwrap();
    assert_eq!(cpu.simd[25], u32x4([0, 0, u32::MAX, 0]));

    cpu.simd[31] = i64x2([-3, 4]);
    cpu.simd[30] = i64x2([-3, 5]);
    execute(&mut cpu, &mut bus, decode(0x4EFE_3FE3).unwrap()).unwrap();
    assert_eq!(cpu.simd[3], u64x2([u64::MAX, 0]));

    execute(&mut cpu, &mut bus, decode(0x4EFE_37E3).unwrap()).unwrap();
    assert_eq!(cpu.simd[3], u64x2([0, 0]));
}
