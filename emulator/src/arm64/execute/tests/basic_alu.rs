use super::*;

#[test]
fn add_x0_x1_x2() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.set_x(1, 10);
    cpu.regs.set_x(2, 32);
    execute(&mut cpu, &mut bus, decode(0x8B02_0020).unwrap()).unwrap();
    assert_eq!(cpu.regs.x(0), 42);
}

#[test]
fn sub_x0_x1_x2() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.set_x(1, 50);
    cpu.regs.set_x(2, 8);
    execute(&mut cpu, &mut bus, decode(0xCB02_0020).unwrap()).unwrap();
    assert_eq!(cpu.regs.x(0), 42);
}

#[test]
fn sbc_xzr_xzr_builds_unsigned_borrow_mask() {
    let (mut cpu, mut bus) = setup();

    cpu.pstate.set_nzcv(true, false, false, false);
    execute(&mut cpu, &mut bus, decode(0xDA1F_03E0).unwrap()).unwrap();
    assert_eq!(cpu.regs.x(0), u64::MAX);

    cpu.pstate.set_nzcv(false, false, true, false);
    execute(&mut cpu, &mut bus, decode(0xDA1F_03E0).unwrap()).unwrap();
    assert_eq!(cpu.regs.x(0), 0);
}

#[test]
fn crc32_scalar_forms_update_w_destination() {
    let (mut cpu, mut bus) = setup();

    cpu.regs.set_x(2, 0xffff_ffff_1234_5678);
    cpu.regs.set_x(5, 0xffff_ffff_ffff_00ab);
    execute(&mut cpu, &mut bus, decode(0x1AC5_4042).unwrap()).unwrap(); // crc32b w2, w2, w5
    assert_eq!(cpu.regs.x(2), 0x1fc8_b738);

    cpu.regs.set_x(2, 0xffff_ffff_1234_5678);
    cpu.regs.set_x(5, 0xffff_ffff_0000_cdef);
    execute(&mut cpu, &mut bus, decode(0x1AC5_4442).unwrap()).unwrap(); // crc32h w2, w2, w5
    assert_eq!(cpu.regs.x(2), 0x59dd_4425);

    cpu.regs.set_x(2, 0xffff_ffff_1234_5678);
    cpu.regs.set_x(3, 0xffff_ffff_89ab_cdef);
    execute(&mut cpu, &mut bus, decode(0x1AC3_4842).unwrap()).unwrap(); // crc32w w2, w2, w3
    assert_eq!(cpu.regs.x(2), 0x40d5_5215);

    cpu.regs.set_x(2, 0xffff_ffff_1234_5678);
    cpu.regs.set_x(4, 0x0123_4567_89ab_cdef);
    execute(&mut cpu, &mut bus, decode(0x9AC4_4C42).unwrap()).unwrap(); // crc32x w2, w2, x4
    assert_eq!(cpu.regs.x(2), 0x9b62_eadf);

    cpu.regs.set_x(2, 0xffff_ffff_1234_5678);
    cpu.regs.set_x(5, 0xffff_ffff_ffff_00ab);
    execute(&mut cpu, &mut bus, decode(0x1AC5_5042).unwrap()).unwrap(); // crc32cb w2, w2, w5
    assert_eq!(cpu.regs.x(2), 0xc091_2609);

    cpu.regs.set_x(2, 0xffff_ffff_1234_5678);
    cpu.regs.set_x(5, 0xffff_ffff_0000_cdef);
    execute(&mut cpu, &mut bus, decode(0x1AC5_5442).unwrap()).unwrap(); // crc32ch w2, w2, w5
    assert_eq!(cpu.regs.x(2), 0xb54a_8725);

    cpu.regs.set_x(2, 0xffff_ffff_1234_5678);
    cpu.regs.set_x(3, 0xffff_ffff_89ab_cdef);
    execute(&mut cpu, &mut bus, decode(0x1AC3_5842).unwrap()).unwrap(); // crc32cw w2, w2, w3
    assert_eq!(cpu.regs.x(2), 0xa360_621e);

    cpu.regs.set_x(2, 0xffff_ffff_1234_5678);
    cpu.regs.set_x(4, 0x0123_4567_89ab_cdef);
    execute(&mut cpu, &mut bus, decode(0x9AC4_5C42).unwrap()).unwrap(); // crc32cx w2, w2, x4
    assert_eq!(cpu.regs.x(2), 0xa3d2_07be);
}
