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

#[test]
fn simd_q_same_page_load_reads_full_vector() {
    let (mut cpu, mut bus) = setup();
    let base = RAM_BASE + 0x6900;
    let low = 0x0102_0304_0506_0708;
    let high = 0x8877_6655_4433_2211;

    bus.mem.write(base, 8, low);
    bus.mem.write(base + 8, 8, high);
    cpu.regs.set_x(4, base);

    execute(&mut cpu, &mut bus, decode(0x3DC0_0084).unwrap()).unwrap(); // ldr q4, [x4]

    assert_eq!(cpu.simd[4], ((high as u128) << 64) | low as u128);
}

#[test]
fn simd_q_cross_page_load_translates_second_page() {
    let (mut cpu, mut bus) = setup();
    let va = 0x1ff8;
    let first_pa = RAM_BASE + 0x0100_0000;
    let second_pa = RAM_BASE + 0x0200_0000;
    let low = 0x0102_0304_0506_0708;
    let high = 0x8877_6655_4433_2211;

    map_two_user_pages(&mut cpu, &mut bus, 0x1000, first_pa, second_pa);
    bus.mem.write(first_pa + 0xff8, 8, low);
    bus.mem.write(second_pa, 8, high);
    cpu.regs.set_x(4, va);

    execute(&mut cpu, &mut bus, decode(0x3DC0_0084).unwrap()).unwrap(); // ldr q4, [x4]

    assert_eq!(cpu.simd[4], ((high as u128) << 64) | low as u128);
}
