use super::simd_helpers::*;
use super::*;

#[test]
fn simd_unsigned_widen_multiply_vectors_select_source_half() {
    let (mut cpu, mut bus) = setup();

    cpu.simd[2] = u16x8([1, 2, 3, 4, 5, 6, 7, 8]);
    cpu.simd[3] = u16x8([10, 20, 30, 40, 50, 60, 70, 80]);
    execute(&mut cpu, &mut bus, decode(0x6E63_C041).unwrap()).unwrap();
    assert_eq!(cpu.simd[1], u32x4([250, 360, 490, 640]));

    cpu.simd[7] = u32x4([2, 3, 5, 7]);
    cpu.simd[6] = u32x4([11, 13, 17, 19]);
    execute(&mut cpu, &mut bus, decode(0x2EA6_C0E5).unwrap()).unwrap();
    assert_eq!(cpu.simd[5], u64x2([22, 39]));

    execute(&mut cpu, &mut bus, decode(0x6EA6_C0E5).unwrap()).unwrap();
    assert_eq!(cpu.simd[5], u64x2([85, 133]));

    cpu.simd[14] = u32x4([2, 3, 5, 7]);
    cpu.simd[7] = u32x4([11, 13, 17, 19]);
    execute(&mut cpu, &mut bus, decode(0x2F87_A9D7).unwrap()).unwrap();
    assert_eq!(cpu.simd[23], u64x2([34, 51]));

    execute(&mut cpu, &mut bus, decode(0x6F87_A9D7).unwrap()).unwrap();
    assert_eq!(cpu.simd[23], u64x2([85, 119]));
}

#[test]
fn simd_unsigned_widen_multiply_accumulate_vectors_wrap_lanes() {
    let (mut cpu, mut bus) = setup();

    cpu.simd[6] = u64x2([u64::MAX - 1, 100]);
    cpu.simd[7] = u32x4([3, 4, 5, 6]);
    execute(&mut cpu, &mut bus, decode(0x2EA7_80E6).unwrap()).unwrap();
    assert_eq!(cpu.simd[6], u64x2([7, 116]));

    cpu.simd[6] = u64x2([10, u64::MAX - 3]);
    cpu.simd[7] = u32x4([3, 4, 5, 6]);
    execute(&mut cpu, &mut bus, decode(0x6EA7_80E6).unwrap()).unwrap();
    assert_eq!(cpu.simd[6], u64x2([35, 32]));
}

fn u16x8(values: [u16; 8]) -> u128 {
    values
        .into_iter()
        .enumerate()
        .fold(0u128, |bits, (lane, value)| {
            bits | ((value as u128) << (lane * 16))
        })
}
