use super::*;

#[test]
fn simd_scalar_byte_halfword_load_store_forms() {
    let (mut cpu, mut bus) = setup();
    let base = RAM_BASE + 0x3800;

    cpu.regs.set_x(19, base);
    cpu.simd[31] = 0x1234_5678_9abc_def0;
    execute(&mut cpu, &mut bus, decode(0x3D01_B67F).unwrap()).unwrap(); // str b31, [x19, #0x6d]
    assert_eq!(bus.read(base + 0x6d, 1), Some(0xf0));

    bus.write(base + 0x80, 1, 0x5a);
    cpu.regs.set_x(2, base + 0x80);
    cpu.simd[31] = u128::MAX;
    execute(&mut cpu, &mut bus, decode(0x3D40_005F).unwrap()).unwrap(); // ldr b31, [x2]
    assert_eq!(cpu.simd[31], 0x5a);

    bus.write(base + 0x90, 2, 0xbeef);
    cpu.regs.set_x(0, base + 0x90);
    execute(&mut cpu, &mut bus, decode(0x7D40_001E).unwrap()).unwrap(); // ldr h30, [x0]
    assert_eq!(cpu.simd[30], 0xbeef);

    cpu.regs.set_x(0, base + 0xa0);
    cpu.simd[30] = 0xabcd;
    execute(&mut cpu, &mut bus, decode(0x7C00_241E).unwrap()).unwrap(); // str h30, [x0], #2
    assert_eq!(bus.read(base + 0xa0, 2), Some(0xabcd));
    assert_eq!(cpu.regs.x(0), base + 0xa2);

    bus.write(base + 0xc0, 1, 0x5a);
    cpu.regs.set_x(3, base + 0xc0);
    cpu.simd[30] = 0x1122_3344_5566_7788;
    execute(&mut cpu, &mut bus, decode(0x0D40_0C7E).unwrap()).unwrap(); // ld1 {v30.b}[3], [x3]
    assert_eq!(cpu.simd[30], 0x1122_3344_5a66_7788);

    cpu.regs.set_x(19, base + 0xc8);
    cpu.simd[28] = (0xbeefu128 << 32) | 0x5555_4444;
    execute(&mut cpu, &mut bus, decode(0x0D00_527C).unwrap()).unwrap(); // st1 {v28.h}[2], [x19]
    assert_eq!(bus.read(base + 0xc8, 2), Some(0xbeef));

    cpu.regs.set_x(8, base + 0xd0);
    cpu.simd[30] = 0x8070_6050_4030_2010;
    execute(&mut cpu, &mut bus, decode(0x0D00_1D1E).unwrap()).unwrap(); // st1 {v30.b}[7], [x8]
    assert_eq!(bus.read(base + 0xd0, 1), Some(0x80));

    cpu.regs.set_x(7, base + 0xe0);
    cpu.simd[6] = 0x1122_3344_5566_7788;
    execute(&mut cpu, &mut bus, decode(0x0D9F_80E6).unwrap()).unwrap(); // st1 {v6.s}[0], [x7], #4
    assert_eq!(bus.read(base + 0xe0, 4), Some(0x5566_7788));
    assert_eq!(cpu.regs.x(7), base + 0xe4);

    cpu.regs.set_x(7, base + 0xe8);
    cpu.regs.set_x(4, 12);
    cpu.simd[6] = 0xaabb_ccdd_0102_0304;
    execute(&mut cpu, &mut bus, decode(0x0D84_80E6).unwrap()).unwrap(); // st1 {v6.s}[0], [x7], x4
    assert_eq!(bus.read(base + 0xe8, 4), Some(0x0102_0304));
    assert_eq!(cpu.regs.x(7), base + 0xf4);

    bus.write(base + 0x100, 4, 0xc001_d00d);
    cpu.regs.set_x(7, base + 0x100);
    cpu.regs.set_x(4, 20);
    cpu.simd[6] = 0xfeed_face_1122_3344;
    execute(&mut cpu, &mut bus, decode(0x0DC4_80E6).unwrap()).unwrap(); // ld1 {v6.s}[0], [x7], x4
    assert_eq!(cpu.simd[6], 0xfeed_face_c001_d00d);
    assert_eq!(cpu.regs.x(7), base + 0x114);

    cpu.regs.set_x(1, base + 0xb0);
    cpu.regs.set_x(0, 3);
    bus.write(base + 0xb6, 2, 0xcafe);
    execute(&mut cpu, &mut bus, decode(0x7C60_783C).unwrap()).unwrap(); // ldr h28, [x1, x0, lsl #1]
    assert_eq!(cpu.simd[28], 0xcafe);

    cpu.simd[28] = 0xface;
    execute(&mut cpu, &mut bus, decode(0x7C20_783C).unwrap()).unwrap(); // str h28, [x1, x0, lsl #1]
    assert_eq!(bus.read(base + 0xb6, 2), Some(0xface));
}

#[test]
fn simd_rcpc_unscaled_load_store_forms_transfer_values() {
    let (mut cpu, mut bus) = setup();
    let base = RAM_BASE + 0x5400;

    cpu.regs.set_x(1, base);
    bus.write(base, 1, 0x5a);
    execute(&mut cpu, &mut bus, decode(0x1D40_0820).unwrap()).unwrap(); // ldapur b0, [x1]
    assert_eq!(cpu.simd[0], 0x5a);

    let low = 0x1122_3344_5566_7788;
    let high = 0x99aa_bbcc_ddee_ff00;
    cpu.regs.set_x(4, base + 0x30);
    bus.write(base + 0x34, 8, low);
    bus.write(base + 0x3c, 8, high);
    execute(&mut cpu, &mut bus, decode(0x1DC0_4884).unwrap()).unwrap(); // ldapur q4, [x4, #4]
    assert_eq!(cpu.simd[4], ((high as u128) << 64) | low as u128);

    cpu.regs.set_x(2, base + 0x80);
    cpu.simd[1] = 0xcafe;
    execute(&mut cpu, &mut bus, decode(0x5D00_1841).unwrap()).unwrap(); // stlur h1, [x2, #1]
    assert_eq!(bus.read(base + 0x81, 2), Some(0xcafe));

    let q_low = 0x0102_0304_0506_0708;
    let q_high = 0x8877_6655_4433_2211;
    cpu.simd[4] = ((q_high as u128) << 64) | q_low as u128;
    execute(&mut cpu, &mut bus, decode(0x1D80_4884).unwrap()).unwrap(); // stlur q4, [x4, #4]
    assert_eq!(bus.read(base + 0x34, 8), Some(q_low));
    assert_eq!(bus.read(base + 0x3c, 8), Some(q_high));
}

#[test]
fn simd_rcpc_single_lane_forms_transfer_doubleword_lanes() {
    let (mut cpu, mut bus) = setup();
    let base = RAM_BASE + 0x5a00;

    cpu.regs.set_x(1, base);
    cpu.simd[0] = 0xaaaa_bbbb_cccc_dddd_1111_2222_3333_4444;
    bus.write(base, 8, 0x0123_4567_89ab_cdef);
    execute(&mut cpu, &mut bus, decode(0x0D41_8420).unwrap()).unwrap(); // ldap1 {v0.d}[0], [x1]
    assert_eq!(cpu.simd[0], 0xaaaa_bbbb_cccc_dddd_0123_4567_89ab_cdef);

    bus.write(base, 8, 0xfedc_ba98_7654_3210);
    execute(&mut cpu, &mut bus, decode(0x4D41_8420).unwrap()).unwrap(); // ldap1 {v0.d}[1], [x1]
    assert_eq!(cpu.simd[0], 0xfedc_ba98_7654_3210_0123_4567_89ab_cdef);

    cpu.regs.set_x(3, base + 0x20);
    cpu.simd[2] = 0x8877_6655_4433_2211_0102_0304_0506_0708;
    execute(&mut cpu, &mut bus, decode(0x0D01_8462).unwrap()).unwrap(); // stl1 {v2.d}[0], [x3]
    assert_eq!(bus.read(base + 0x20, 8), Some(0x0102_0304_0506_0708));
    execute(&mut cpu, &mut bus, decode(0x4D01_8462).unwrap()).unwrap(); // stl1 {v2.d}[1], [x3]
    assert_eq!(bus.read(base + 0x20, 8), Some(0x8877_6655_4433_2211));
}
