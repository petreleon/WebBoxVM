use super::simd_helpers::*;
use super::*;

#[test]
fn simd_compare_zero_forms_use_signed_lane_ordering() {
    let (mut cpu, mut bus) = setup();

    cpu.simd[1] = i32x4([-1, 0, 5, i32::MIN]);
    execute(&mut cpu, &mut bus, decode(0x4EA0_8821).unwrap()).unwrap();
    assert_eq!(cpu.simd[1], u32x4([0, 0, u32::MAX, 0]));

    cpu.simd[1] = i32x4([-1, 0, 5, i32::MIN]);
    execute(&mut cpu, &mut bus, decode(0x4EA0_A821).unwrap()).unwrap();
    assert_eq!(cpu.simd[1], u32x4([u32::MAX, 0, 0, u32::MAX]));

    cpu.simd[1] = i32x4([-1, 0, 5, i32::MIN]);
    execute(&mut cpu, &mut bus, decode(0x6EA0_9821).unwrap()).unwrap();
    assert_eq!(cpu.simd[1], u32x4([u32::MAX, u32::MAX, 0, u32::MAX]));

    cpu.simd[1] = i64x2([-1, 9]);
    execute(&mut cpu, &mut bus, decode(0x5EE0_8823).unwrap()).unwrap();
    assert_eq!(cpu.simd[3], 0);
}
