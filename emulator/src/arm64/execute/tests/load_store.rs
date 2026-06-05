use super::*;

#[test]
fn ldp_loads_pair() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.set_x(1, 0x4000_0000);
    bus.mem.write(0x4000_0000, 8, 0xDEAD_BEEF);
    bus.mem.write(0x4000_0008, 8, 0xCAFE_BABE);
    execute(&mut cpu, &mut bus, decode(0xA940_0C22).unwrap()).unwrap();
    assert_eq!(cpu.regs.x(2), 0xDEAD_BEEF);
    assert_eq!(cpu.regs.x(3), 0xCAFE_BABE);
}

#[test]
fn scalar_load_store_translate_each_page_when_crossing_boundary() {
    let (mut cpu, mut bus) = setup();
    let va = 0x1ffc;
    let first_pa = RAM_BASE + 0x0100_0000;
    let second_pa = RAM_BASE + 0x0200_0000;
    map_two_user_pages(&mut cpu, &mut bus, 0x1000, first_pa, second_pa);

    bus.mem.write(first_pa + 0xffc, 4, 0x5566_7788);
    bus.mem.write(second_pa, 4, 0x1122_3344);
    cpu.regs.set_x(1, va);
    execute(&mut cpu, &mut bus, decode(0xF940_0022).unwrap()).unwrap(); // ldr x2, [x1]
    assert_eq!(cpu.regs.x(2), 0x1122_3344_5566_7788);

    bus.mem.write(first_pa + PAGE_SIZE, 4, 0xDEAD_BEEF);
    cpu.regs.set_x(0, 0xAABB_CCDD_EEFF_0011);
    execute(&mut cpu, &mut bus, decode(0xF900_0020).unwrap()).unwrap(); // str x0, [x1]

    assert_eq!(bus.mem.read(first_pa + 0xffc, 4), Some(0xEEFF_0011));
    assert_eq!(bus.mem.read(second_pa, 4), Some(0xAABB_CCDD));
    assert_eq!(bus.mem.read(first_pa + PAGE_SIZE, 4), Some(0xDEAD_BEEF));
}

#[test]
fn ldpsw_loads_and_sign_extends_pair() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.set_x(19, 0x4000_0000);
    bus.mem.write(0x4000_0064, 4, 0xffff_fffc);
    bus.mem.write(0x4000_0068, 4, 0x7fff_fffe);

    execute(&mut cpu, &mut bus, decode(0x694C_9262).unwrap()).unwrap();

    assert_eq!(cpu.regs.x(2), 0xffff_ffff_ffff_fffc);
    assert_eq!(cpu.regs.x(4), 0x7fff_fffe);
}

#[test]
fn authenticated_loads_read_doubleword_and_preindex_writeback() {
    let (mut cpu, mut bus) = setup();
    let base = 0x4000_0200;
    bus.mem.write(base - 128, 8, 0x1122_3344_5566_7788);
    cpu.regs.set_x(16, base);

    execute(&mut cpu, &mut bus, decode(0xF8FF_060D).unwrap()).unwrap();

    assert_eq!(cpu.regs.x(13), 0x1122_3344_5566_7788);
    assert_eq!(cpu.regs.x(16), base);

    bus.mem.write(base + 16, 8, 0xAABB_CCDD_EEFF_0011);
    cpu.regs.set_x(1, base);
    execute(&mut cpu, &mut bus, decode(0xF820_2C20).unwrap()).unwrap();

    assert_eq!(cpu.regs.x(0), 0xAABB_CCDD_EEFF_0011);
    assert_eq!(cpu.regs.x(1), base + 16);
}

#[test]
fn rcpc_unscaled_loads_read_and_extend_like_scalar_loads() {
    let (mut cpu, mut bus) = setup();
    let base = 0x4000_0300;
    cpu.regs.set_x(1, base);

    bus.mem.write(base + 1, 1, 0xAA);
    execute(&mut cpu, &mut bus, decode(0x1940_1022).unwrap()).unwrap();
    assert_eq!(cpu.regs.x(2), 0xAA);

    bus.mem.write(base + 2, 2, 0x1122);
    execute(&mut cpu, &mut bus, decode(0x5940_2023).unwrap()).unwrap();
    assert_eq!(cpu.regs.x(3), 0x1122);

    bus.mem.write(base + 3, 4, 0x5566_7788);
    execute(&mut cpu, &mut bus, decode(0x9940_3024).unwrap()).unwrap();
    assert_eq!(cpu.regs.x(4), 0x5566_7788);

    bus.mem.write(base + 4, 8, 0x8877_6655_4433_2211);
    execute(&mut cpu, &mut bus, decode(0xD940_4025).unwrap()).unwrap();
    assert_eq!(cpu.regs.x(5), 0x8877_6655_4433_2211);

    bus.mem.write(base + 5, 1, 0x80);
    execute(&mut cpu, &mut bus, decode(0x19C0_5026).unwrap()).unwrap();
    assert_eq!(cpu.regs.x(6), 0xFFFF_FF80);

    bus.mem.write(base + 6, 2, 0x8001);
    execute(&mut cpu, &mut bus, decode(0x5980_6027).unwrap()).unwrap();
    assert_eq!(cpu.regs.x(7), 0xFFFF_FFFF_FFFF_8001);

    bus.mem.write(base + 7, 4, 0xFFFF_FF80);
    execute(&mut cpu, &mut bus, decode(0x9980_7028).unwrap()).unwrap();
    assert_eq!(cpu.regs.x(8), 0xFFFF_FFFF_FFFF_FF80);

    bus.mem.write(base - 1, 2, 0xCAFE);
    execute(&mut cpu, &mut bus, decode(0x595F_F029).unwrap()).unwrap();
    assert_eq!(cpu.regs.x(9), 0xCAFE);
}

#[test]
fn simd_pair_store_then_load_roundtrips_vectors() {
    let (mut cpu, mut bus) = setup();
    let base = 0x4000_0100;

    cpu.regs.set_x(6, base);
    cpu.simd[31] = 0x0011_2233_4455_6677_8899_aabb_ccdd_eeff;
    cpu.simd[30] = 0xffee_ddcc_bbaa_9988_7766_5544_3322_1100;
    execute(&mut cpu, &mut bus, decode(0xAC81_78DF).unwrap()).unwrap();
    assert_eq!(cpu.regs.x(6), base + 32);

    cpu.simd[29] = 0;
    cpu.simd[28] = 0;
    cpu.regs.set_x(6, base);
    execute(&mut cpu, &mut bus, decode(0xAD40_70DD).unwrap()).unwrap();
    assert_eq!(cpu.simd[29], 0x0011_2233_4455_6677_8899_aabb_ccdd_eeff);
    assert_eq!(cpu.simd[28], 0xffee_ddcc_bbaa_9988_7766_5544_3322_1100);
}
