use super::*;

#[test]
fn casa_updates_memory_on_match_and_returns_old() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.set_x(0, 0x4000_0000);
    cpu.regs.set_w(1, 0x1111_2222);
    cpu.regs.set_w(2, 0x3333_4444);
    bus.mem.write(0x4000_0000, 4, 0x1111_2222);

    execute(&mut cpu, &mut bus, decode(0x88E1_7C02).unwrap()).unwrap();

    assert_eq!(bus.mem.read(0x4000_0000, 4), Some(0x3333_4444));
    assert_eq!(cpu.regs.x(1), 0x1111_2222);
}

#[test]
fn caspal_updates_pair_on_match_and_returns_old_pair() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.set_x(4, 0x4000_0000);
    cpu.regs.set_x(0, 0x1111_2222_3333_4444);
    cpu.regs.set_x(1, 0x5555_6666_7777_8888);
    cpu.regs.set_x(2, 0xAAAA_BBBB_CCCC_DDDD);
    cpu.regs.set_x(3, 0xEEEE_FFFF_0000_1111);
    bus.mem.write(0x4000_0000, 8, 0x1111_2222_3333_4444);
    bus.mem.write(0x4000_0008, 8, 0x5555_6666_7777_8888);

    execute(&mut cpu, &mut bus, decode(0x4860_FC82).unwrap()).unwrap();

    assert_eq!(bus.mem.read(0x4000_0000, 8), Some(0xAAAA_BBBB_CCCC_DDDD));
    assert_eq!(bus.mem.read(0x4000_0008, 8), Some(0xEEEE_FFFF_0000_1111));
    assert_eq!(cpu.regs.x(0), 0x1111_2222_3333_4444);
    assert_eq!(cpu.regs.x(1), 0x5555_6666_7777_8888);
}

#[test]
fn ldaddal_adds_and_returns_old() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.set_x(0, 0x4000_0000);
    cpu.regs.set_w(1, 5);
    bus.mem.write(0x4000_0000, 4, 7);

    execute(&mut cpu, &mut bus, decode(0xB8E1_0001).unwrap()).unwrap();

    assert_eq!(bus.mem.read(0x4000_0000, 4), Some(12));
    assert_eq!(cpu.regs.x(1), 7);
}

#[test]
fn ldseta_sets_bits_and_returns_old() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.set_x(19, 0x4000_0000);
    cpu.regs.set_x(0, 0b1010);
    bus.mem.write(0x4000_0000, 8, 0b0101);

    execute(&mut cpu, &mut bus, decode(0xF8A0_3260).unwrap()).unwrap();

    assert_eq!(bus.mem.read(0x4000_0000, 8), Some(0b1111));
    assert_eq!(cpu.regs.x(0), 0b0101);
}

#[test]
fn lse128_pair_atomics_update_quadword_and_return_old() {
    let (mut cpu, mut bus) = setup();

    cpu.regs.set_x(6, 0x4000_0000);
    cpu.regs.set_x(0, 0xAAAA_BBBB_CCCC_DDDD);
    cpu.regs.set_x(1, 0xEEEE_FFFF_0000_1111);
    bus.mem.write(0x4000_0000, 8, 0x1111_2222_3333_4444);
    bus.mem.write(0x4000_0008, 8, 0x5555_6666_7777_8888);

    execute(&mut cpu, &mut bus, decode(0x1921_80C0).unwrap()).unwrap(); // swpp x0, x1, [x6]

    assert_eq!(bus.mem.read(0x4000_0000, 8), Some(0xAAAA_BBBB_CCCC_DDDD));
    assert_eq!(bus.mem.read(0x4000_0008, 8), Some(0xEEEE_FFFF_0000_1111));
    assert_eq!(cpu.regs.x(0), 0x1111_2222_3333_4444);
    assert_eq!(cpu.regs.x(1), 0x5555_6666_7777_8888);

    cpu.regs.set_x(0, 0x4000_0020);
    cpu.regs.set_x(2, 0x00FF_0000_00FF_0000);
    cpu.regs.set_x(3, 0x0000_FF00_0000_FF00);
    bus.mem.write(0x4000_0020, 8, 0x1100_1100_1100_1100);
    bus.mem.write(0x4000_0028, 8, 0x2200_2200_2200_2200);

    execute(&mut cpu, &mut bus, decode(0x19A3_3002).unwrap()).unwrap(); // ldsetpa x2, x3, [x0]

    assert_eq!(bus.mem.read(0x4000_0020, 8), Some(0x11FF_1100_11FF_1100));
    assert_eq!(bus.mem.read(0x4000_0028, 8), Some(0x2200_FF00_2200_FF00));
    assert_eq!(cpu.regs.x(2), 0x1100_1100_1100_1100);
    assert_eq!(cpu.regs.x(3), 0x2200_2200_2200_2200);

    cpu.regs.set_x(5, 0x4000_0040);
    cpu.regs.set_x(6, 0x00FF_0000_00FF_0000);
    cpu.regs.set_x(7, 0x0000_FF00_0000_FF00);
    bus.mem.write(0x4000_0040, 8, 0xFFFF_FFFF_FFFF_FFFF);
    bus.mem.write(0x4000_0048, 8, 0xFFFF_FFFF_FFFF_FFFF);

    execute(&mut cpu, &mut bus, decode(0x19E7_10A6).unwrap()).unwrap(); // ldclrpal x6, x7, [x5]

    assert_eq!(bus.mem.read(0x4000_0040, 8), Some(0xFF00_FFFF_FF00_FFFF));
    assert_eq!(bus.mem.read(0x4000_0048, 8), Some(0xFFFF_00FF_FFFF_00FF));
    assert_eq!(cpu.regs.x(6), 0xFFFF_FFFF_FFFF_FFFF);
    assert_eq!(cpu.regs.x(7), 0xFFFF_FFFF_FFFF_FFFF);
}

#[test]
fn ldxp_stlxp_pair_roundtrip() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.set_x(2, 0x4000_0000);
    cpu.regs.set_x(0, 0xAAAA);
    cpu.regs.set_x(1, 0xBBBB);
    cpu.reserve_exclusive(0x4000_0000, 16);

    execute(&mut cpu, &mut bus, decode(0xC823_8440).unwrap()).unwrap();

    assert_eq!(bus.mem.read(0x4000_0000, 8), Some(0xAAAA));
    assert_eq!(bus.mem.read(0x4000_0008, 8), Some(0xBBBB));
    assert_eq!(cpu.regs.x(3), 0);

    cpu.regs.set_x(0, 0);
    cpu.regs.set_x(1, 0);
    execute(&mut cpu, &mut bus, decode(0xC87F_8440).unwrap()).unwrap();

    assert_eq!(cpu.regs.x(0), 0xAAAA);
    assert_eq!(cpu.regs.x(1), 0xBBBB);
}

#[test]
fn rcpc3_gpr_writeback_load_store_forms() {
    let (mut cpu, mut bus) = setup();
    let base = RAM_BASE + 0x5c00;

    cpu.regs.set_x(1, base);
    bus.mem.write(base, 4, 0xaabb_ccdd);
    execute(&mut cpu, &mut bus, decode(0x99C0_0820).unwrap()).unwrap(); // ldapr w0, [x1], #4
    assert_eq!(cpu.regs.x(0), 0xaabb_ccdd);
    assert_eq!(cpu.regs.x(1), base + 4);

    cpu.regs.set_x(3, base + 0x20);
    bus.mem.write(base + 0x20, 8, 0x1122_3344_5566_7788);
    execute(&mut cpu, &mut bus, decode(0xD9C0_0862).unwrap()).unwrap(); // ldapr x2, [x3], #8
    assert_eq!(cpu.regs.x(2), 0x1122_3344_5566_7788);
    assert_eq!(cpu.regs.x(3), base + 0x28);

    cpu.regs.set_x(5, base + 0x44);
    cpu.regs.set_w(4, 0x1357_9bdf);
    execute(&mut cpu, &mut bus, decode(0x9980_08A4).unwrap()).unwrap(); // stlr w4, [x5, #-4]!
    assert_eq!(cpu.regs.x(5), base + 0x40);
    assert_eq!(bus.mem.read(base + 0x40, 4), Some(0x1357_9bdf));

    cpu.regs.set_x(7, base + 0x70);
    cpu.regs.set_x(6, 0x8877_6655_4433_2211);
    execute(&mut cpu, &mut bus, decode(0xD980_08E6).unwrap()).unwrap(); // stlr x6, [x7, #-8]!
    assert_eq!(cpu.regs.x(7), base + 0x68);
    assert_eq!(bus.mem.read(base + 0x68, 8), Some(0x8877_6655_4433_2211));
}
