use super::*;

#[test]
fn mov_reg_copies_value() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.set_x(1, 0x1234_5678);
    execute(&mut cpu, &mut bus, decode(0xAA01_03E0).unwrap()).unwrap();
    assert_eq!(cpu.regs.x(0), 0x1234_5678);
}

#[test]
fn add_imm_adds_constant() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.set_x(1, 10);
    execute(&mut cpu, &mut bus, decode(0x9100_0420).unwrap()).unwrap();
    assert_eq!(cpu.regs.x(0), 11);
}

#[test]
fn addsub_shifted_register_rd31_is_zero_register() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.sp = 0x8000;
    cpu.regs.set_x(1, 10);
    cpu.regs.set_x(2, 3);

    execute(&mut cpu, &mut bus, decode(0x8B02_003F).unwrap()).unwrap();
    assert_eq!(cpu.regs.sp, 0x8000);

    execute(&mut cpu, &mut bus, decode(0xCB02_003F).unwrap()).unwrap();
    assert_eq!(cpu.regs.sp, 0x8000);
}

#[test]
fn movk_merges_value() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.set_x(0, 0xDEAD_BEEF_0000_0000);
    execute(&mut cpu, &mut bus, decode(0xF282_4680).unwrap()).unwrap();
    assert_eq!(cpu.regs.x(0), 0xDEAD_BEEF_0000_1234);
}

#[test]
fn adrp_sets_page_relative() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.pc = 0x4000_0400;
    execute(&mut cpu, &mut bus, decode(0x9000_0000).unwrap()).unwrap();
    assert_eq!(cpu.regs.x(0), 0x4000_0000);
}

#[test]
fn tbz_branches_when_bit_clear() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.pc = 0x4000_0000;
    cpu.regs.set_x(0, 0b110);
    execute(&mut cpu, &mut bus, decode(0x3600_0020).unwrap()).unwrap();
    assert_eq!(cpu.regs.pc, 0x4000_0004);
}

#[test]
fn cmp_sets_flags() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.set_x(2, 10);
    cpu.regs.set_x(3, 5);
    execute(&mut cpu, &mut bus, decode(0xEB02007F).unwrap()).unwrap();
    assert!(!cpu.pstate.z());
    assert!(cpu.pstate.n());
}

#[test]
fn cmp_equal_sets_z() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.set_x(2, 5);
    cpu.regs.set_x(3, 5);
    execute(&mut cpu, &mut bus, decode(0xEB02007F).unwrap()).unwrap();
    assert!(cpu.pstate.z());
    assert!(!cpu.pstate.n());
}

#[test]
fn cmp_less_than_sets_n() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.set_x(2, 3);
    cpu.regs.set_x(3, 10);
    execute(&mut cpu, &mut bus, decode(0xEB02007F).unwrap()).unwrap();
    assert!(!cpu.pstate.n());
    assert!(!cpu.pstate.z());
}

#[test]
fn cmp_extended_uses_sp_base() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.sp = 0x4000_0000;
    cpu.regs.set_x(2, 0x4000_0000);

    execute(&mut cpu, &mut bus, decode(0xEB22_63FF).unwrap()).unwrap();

    assert!(cpu.pstate.z());
}

#[test]
fn cmp_immediate_uses_sp_base() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.sp = 0x10;

    execute(&mut cpu, &mut bus, decode(0xF100_43FF).unwrap()).unwrap();

    assert!(cpu.pstate.z());
}

#[test]
fn logical_immediates_apply_decoded_masks() {
    let (mut cpu, mut bus) = setup();

    cpu.regs.set_x(1, 0x1234);
    execute(&mut cpu, &mut bus, decode(0x9240_1C20).unwrap()).unwrap();
    assert_eq!(cpu.regs.x(0), 0x34);

    cpu.regs.set_x(3, 0x12);
    execute(&mut cpu, &mut bus, decode(0xB278_1C62).unwrap()).unwrap();
    assert_eq!(cpu.regs.x(2), 0xff12);

    cpu.regs.set_w(5, 0xffff_0000);
    execute(&mut cpu, &mut bus, decode(0x5200_9CA4).unwrap()).unwrap();
    assert_eq!(cpu.regs.x(4), 0xff00_00ff);

    cpu.regs.set_x(7, 0x0f0f_0f0f_0f0f_0f0f);
    execute(&mut cpu, &mut bus, decode(0xF204_CCE6).unwrap()).unwrap();
    assert_eq!(cpu.regs.x(6), 0);
    assert!(cpu.pstate.z());
    assert!(!cpu.pstate.n());
}

#[test]
fn addsub_immediates_update_values_and_flags() {
    let (mut cpu, mut bus) = setup();

    cpu.regs.set_x(1, 0x1000);
    execute(&mut cpu, &mut bus, decode(0x9104_8C20).unwrap()).unwrap();
    assert_eq!(cpu.regs.x(0), 0x1123);

    cpu.regs.set_x(5, 0x100);
    execute(&mut cpu, &mut bus, decode(0xD101_54A4).unwrap()).unwrap();
    assert_eq!(cpu.regs.x(4), 0xab);

    cpu.regs.set_w(7, u32::MAX);
    execute(&mut cpu, &mut bus, decode(0x3100_40E6).unwrap()).unwrap();
    assert_eq!(cpu.regs.x(6), 0x0f);
    assert!(cpu.pstate.c());

    cpu.regs.set_x(8, 0x20);
    execute(&mut cpu, &mut bus, decode(0xF100_811F).unwrap()).unwrap();
    assert!(cpu.pstate.z());
}
