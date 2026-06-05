use super::*;

#[test]
fn simd_q_same_page_store_clears_overlapping_exclusive() {
    let (mut cpu, mut bus) = setup();
    let base = RAM_BASE + 0x6800;
    let low = 0x0102_0304_0506_0708;
    let high = 0x8877_6655_4433_2211;

    cpu.regs.set_x(4, base);
    cpu.simd[4] = ((high as u128) << 64) | low as u128;
    cpu.reserve_exclusive(base + 8, 8);

    execute(&mut cpu, &mut bus, decode(0x3D80_0084).unwrap()).unwrap(); // str q4, [x4]

    assert_eq!(bus.read(base, 8), Some(low));
    assert_eq!(bus.read(base + 8, 8), Some(high));
    assert!(cpu.exclusive.is_none());
}
