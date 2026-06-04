use super::simd_helpers::*;
use super::*;

#[test]
fn simd_minmax_forms_use_signed_and_pairwise_lane_ordering() {
    let (mut cpu, mut bus) = setup();

    cpu.simd[1] = i32x4([-5, 6, i32::MIN, 10]);
    cpu.simd[0] = i32x4([-7, 5, 1, -11]);
    execute(&mut cpu, &mut bus, decode(0x4EA0_6C20).unwrap()).unwrap();
    assert_eq!(cpu.simd[0], i32x4([-7, 5, i32::MIN, -11]));

    cpu.simd[1] = i32x4([-5, 8, 7, -9]);
    cpu.simd[0] = i32x4([3, -2, i32::MIN, 4]);
    execute(&mut cpu, &mut bus, decode(0x4EA0_A420).unwrap()).unwrap();
    assert_eq!(cpu.simd[0], i32x4([8, 7, 3, 4]));

    cpu.simd[1] = i32x4([-5, 8, 7, -9]);
    cpu.simd[0] = i32x4([3, -2, i32::MIN, 4]);
    execute(&mut cpu, &mut bus, decode(0x4EA0_AC20).unwrap()).unwrap();
    assert_eq!(cpu.simd[0], i32x4([-5, -9, -2, i32::MIN]));

    cpu.simd[1] = 0x0807_0605_0403_0201;
    cpu.simd[0] = 0x0102_0304_0506_0708;
    execute(&mut cpu, &mut bus, decode(0x2E20_A420).unwrap()).unwrap();
    assert_eq!(cpu.simd[0], 0x0204_0608_0806_0402);
}
