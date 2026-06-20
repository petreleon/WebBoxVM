use super::simd_helpers::*;
use super::*;

#[test]
fn simd_uqadd_saturates_unsigned_word_lanes() {
    let (mut cpu, mut bus) = setup();
    let qc = 1 << 27;

    cpu.simd[30] = u32x4([1, u32::MAX - 1, u32::MAX, 0x8000_0000]);
    cpu.simd[27] = u32x4([2, 2, 1, 0x8000_0000]);
    execute(&mut cpu, &mut bus, decode(0x6EBB_0FDE).unwrap()).unwrap();

    assert_eq!(cpu.simd[30], u32x4([3, u32::MAX, u32::MAX, u32::MAX]));
    assert_ne!(cpu.sys.fpsr & qc, 0);
}

#[test]
fn simd_uqadd_handles_doubleword_lanes_and_q0_zeroing() {
    let (mut cpu, mut bus) = setup();
    let qc = 1 << 27;

    cpu.simd[28] = u64x2([u64::MAX - 1, 5]);
    cpu.simd[25] = u64x2([2, 6]);
    execute(&mut cpu, &mut bus, decode(0x6EF9_0F9C).unwrap()).unwrap();
    assert_eq!(cpu.simd[28], u64x2([u64::MAX, 11]));
    assert_ne!(cpu.sys.fpsr & qc, 0);

    cpu.sys.fpsr = 0;
    cpu.simd[30] = u32x4([1, u32::MAX - 1, 0xdead_beef, 0xcafe_babe]);
    cpu.simd[27] = u32x4([2, 2, 0x1111_1111, 0x2222_2222]);
    execute(&mut cpu, &mut bus, decode(0x2EBB_0FDE).unwrap()).unwrap();
    assert_eq!(cpu.simd[30], u32x4([3, u32::MAX, 0, 0]));
    assert_ne!(cpu.sys.fpsr & qc, 0);
}
