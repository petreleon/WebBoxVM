use super::*;

#[test]
fn rcpc3_pair_load_store_forms_transfer_and_writeback() {
    let (mut cpu, mut bus) = setup();
    let base = RAM_BASE + 0x6000;

    cpu.regs.set_x(2, base);
    bus.mem.write(base, 4, 0x1122_3344);
    bus.mem.write(base + 4, 4, 0x5566_7788);
    execute(&mut cpu, &mut bus, decode(0x9941_1840).unwrap()).unwrap(); // ldiapp w0, w1, [x2]
    assert_eq!(cpu.regs.x(0), 0x1122_3344);
    assert_eq!(cpu.regs.x(1), 0x5566_7788);
    assert_eq!(cpu.regs.x(2), base);

    cpu.regs.set_x(11, base + 0x20);
    bus.mem.write(base + 0x20, 8, 0x0102_0304_0506_0708);
    bus.mem.write(base + 0x28, 8, 0x8877_6655_4433_2211);
    execute(&mut cpu, &mut bus, decode(0xD94A_0969).unwrap()).unwrap(); // ldiapp x9, x10, [x11], #16
    assert_eq!(cpu.regs.x(9), 0x0102_0304_0506_0708);
    assert_eq!(cpu.regs.x(10), 0x8877_6655_4433_2211);
    assert_eq!(cpu.regs.x(11), base + 0x30);

    cpu.regs.set_x(14, base + 0x50);
    cpu.regs.set_w(12, 0xaabb_ccdd);
    cpu.regs.set_w(13, 0xeeff_0011);
    execute(&mut cpu, &mut bus, decode(0x990D_19CC).unwrap()).unwrap(); // stilp w12, w13, [x14]
    assert_eq!(bus.mem.read(base + 0x50, 4), Some(0xaabb_ccdd));
    assert_eq!(bus.mem.read(base + 0x54, 4), Some(0xeeff_0011));
    assert_eq!(cpu.regs.x(14), base + 0x50);

    cpu.regs.set_x(23, base + 0x90);
    cpu.regs.set_x(21, 0x0123_4567_89ab_cdef);
    cpu.regs.set_x(22, 0xfedc_ba98_7654_3210);
    execute(&mut cpu, &mut bus, decode(0xD916_0AF5).unwrap()).unwrap(); // stilp x21, x22, [x23, #-16]!
    assert_eq!(bus.mem.read(base + 0x80, 8), Some(0x0123_4567_89ab_cdef));
    assert_eq!(bus.mem.read(base + 0x88, 8), Some(0xfedc_ba98_7654_3210));
    assert_eq!(cpu.regs.x(23), base + 0x80);
}

#[test]
fn simd_q_store_faults_without_partial_cross_page_write() {
    let (mut cpu, mut bus) = setup();
    let dst_va = 0x1ff8;
    let first_pa = RAM_BASE + 0x0500_0000;
    let second_pa = RAM_BASE + 0x0600_0000;
    let old = 0xaaaa_aaaa_aaaa_aaaa;
    let low = 0x6974_7069_7263_7365u64;
    let high = 0x4320_3a6e_6f69_7470u64;
    let l3_second_pte = RAM_BASE + 2 * PAGE_SIZE + 2 * 8;

    map_two_user_pages(&mut cpu, &mut bus, 0x1000, first_pa, second_pa);
    bus.mem.write(l3_second_pte, 8, 0);
    bus.write(first_pa + 0xff8, 8, old);
    cpu.regs.set_x(0, 0);
    cpu.regs.set_x(28, dst_va);
    cpu.simd[31] = ((high as u128) << 64) | low as u128;

    let err = execute(&mut cpu, &mut bus, decode(0x3CA0_6B9F).unwrap()).unwrap_err();

    assert_eq!(err, "translation fault");
    assert_eq!(cpu.sys.far_el1, 0x2000);
    assert_eq!(bus.read(first_pa + 0xff8, 8), Some(old));
}
