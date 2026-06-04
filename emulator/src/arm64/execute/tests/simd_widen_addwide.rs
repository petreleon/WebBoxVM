use super::simd_helpers::*;
use super::*;

#[test]
fn simd_add_wide_forms_extend_rhs_and_wrap_destination_lanes() {
    let (mut cpu, mut bus) = setup();

    cpu.simd[29] = i32x4([100, -300, i32::MAX, i32::MIN]);
    cpu.simd[30] = i16x8([1, -2, 3, -4, 100, 101, 102, 103]);
    execute(&mut cpu, &mut bus, decode(0x0E7E_13BD).unwrap()).unwrap();
    assert_eq!(
        cpu.simd[29],
        i32x4([
            101,
            -302,
            i32::MAX.wrapping_add(3),
            i32::MIN.wrapping_sub(4)
        ])
    );

    cpu.simd[31] = i32x4([10, -10, 1000, -1000]);
    cpu.simd[30] = i16x8([1, 2, 3, 4, 5, -6, 7, -8]);
    execute(&mut cpu, &mut bus, decode(0x4E7E_13FF).unwrap()).unwrap();
    assert_eq!(cpu.simd[31], i32x4([15, -16, 1007, -1008]));

    cpu.simd[30] = u64x2([5, u64::MAX - 1]);
    cpu.simd[31] = u32x4([7, 2, 0xffff_ffff, 0]);
    execute(&mut cpu, &mut bus, decode(0x2EBF_13DF).unwrap()).unwrap();
    assert_eq!(cpu.simd[31], u64x2([12, 0]));

    cpu.simd[7] = u64x2([100, u64::MAX]);
    cpu.simd[8] = u32x4([1, 2, 50, 1]);
    execute(&mut cpu, &mut bus, decode(0x6EA8_10E6).unwrap()).unwrap();
    assert_eq!(cpu.simd[6], u64x2([150, 0]));

    cpu.simd[17] = vector_bytes(1);
    cpu.simd[2] = vector_bytes(20);
    execute(&mut cpu, &mut bus, decode(0x2E22_0230).unwrap()).unwrap();
    assert_eq!(cpu.simd[16], u16x8([21, 23, 25, 27, 29, 31, 33, 35]));

    cpu.simd[17] = vector_bytes(1);
    cpu.simd[2] = vector_bytes(20);
    execute(&mut cpu, &mut bus, decode(0x6E22_0231).unwrap()).unwrap();
    assert_eq!(cpu.simd[17], u16x8([37, 39, 41, 43, 45, 47, 49, 51]));

    cpu.simd[22] = u16x8([100, 200, 300, 400, 500, 600, 700, 800]);
    cpu.simd[3] = vector_bytes(1);
    execute(&mut cpu, &mut bus, decode(0x2E23_32D6).unwrap()).unwrap();
    assert_eq!(cpu.simd[22], u16x8([99, 198, 297, 396, 495, 594, 693, 792]));

    cpu.simd[23] = u16x8([1000, 1000, 1000, 1000, 1000, 1000, 1000, 1000]);
    cpu.simd[3] = vector_bytes(1);
    execute(&mut cpu, &mut bus, decode(0x6E23_32F7).unwrap()).unwrap();
    assert_eq!(
        cpu.simd[23],
        u16x8([991, 990, 989, 988, 987, 986, 985, 984])
    );
}

fn i16x8(values: [i16; 8]) -> u128 {
    values
        .into_iter()
        .enumerate()
        .fold(0u128, |bits, (lane, value)| {
            bits | ((value as u16 as u128) << (lane * 16))
        })
}
