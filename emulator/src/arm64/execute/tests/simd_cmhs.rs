use super::simd_helpers::*;
use super::*;

#[test]
fn simd_cmhs_register_compares_unsigned_lane_widths() {
    let (mut cpu, mut bus) = setup();

    cpu.simd[4] = 0x0004_0003_0002_0001;
    cpu.simd[26] = 0x0003_0003_0002_0005;
    execute(&mut cpu, &mut bus, decode(0x2E7A_3C84).unwrap()).unwrap(); // cmhs v4.4h, v4.4h, v26.4h
    assert_eq!(cpu.simd[4], 0xffff_ffff_ffff_0000);

    cpu.simd[25] = u32x4([1, 5, 7, 9]);
    cpu.simd[29] = u32x4([2, 5, 6, 10]);
    execute(&mut cpu, &mut bus, decode(0x6EBD_3F39).unwrap()).unwrap(); // cmhs v25.4s, v25.4s, v29.4s
    assert_eq!(cpu.simd[25], u32x4([0, u32::MAX, u32::MAX, 0]));

    cpu.simd[31] = u64x2([9, 4]);
    cpu.simd[30] = u64x2([8, 5]);
    execute(&mut cpu, &mut bus, decode(0x6EFE_3FE3).unwrap()).unwrap(); // cmhs v3.2d, v31.2d, v30.2d
    assert_eq!(cpu.simd[3], u64x2([u64::MAX, 0]));
}
