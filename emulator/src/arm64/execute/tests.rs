use super::*;
use crate::arm64::decode::decode;

fn setup() -> (Armv8Cpu, SystemBus) {
    (Armv8Cpu::new(), SystemBus::new())
}

#[test]
fn add_x0_x1_x2() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.set_x(1, 10);
    cpu.regs.set_x(2, 32);
    execute(&mut cpu, &mut bus, decode(0x9A02_0020).unwrap()).unwrap();
    assert_eq!(cpu.regs.x(0), 42);
}

#[test]
fn sub_x0_x1_x2() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.set_x(1, 50);
    cpu.regs.set_x(2, 8);
    execute(&mut cpu, &mut bus, decode(0xDA02_0020).unwrap()).unwrap();
    assert_eq!(cpu.regs.x(0), 42);
}

#[test]
fn nop_advances_pc() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.pc = 0x4000_0000;
    execute(&mut cpu, &mut bus, decode(0xD503_201F).unwrap()).unwrap();
    assert_eq!(cpu.regs.pc, 0x4000_0004);
}

#[test]
fn branch_forward_4_bytes() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.pc = 0x4000_0000;
    execute(&mut cpu, &mut bus, decode(0x1400_0002).unwrap()).unwrap();
    assert_eq!(cpu.regs.pc, 0x4000_0008);
}

#[test]
fn bl_sets_lr_and_jumps() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.pc = 0x4000_0000;
    execute(&mut cpu, &mut bus, decode(0x9400_0002).unwrap()).unwrap();
    assert_eq!(cpu.regs.x(30), 0x4000_0004);
    assert_eq!(cpu.regs.pc, 0x4000_0008);
}

#[test]
fn ret_returns_to_lr() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.set_x(30, 0x4000_0100);
    execute(&mut cpu, &mut bus, decode(0xD65F03C0).unwrap()).unwrap();
    assert_eq!(cpu.regs.pc, 0x4000_0100);
}

#[test]
fn cbz_branches_when_zero() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.pc = 0x4000_0000;
    cpu.regs.set_x(0, 0);
    execute(&mut cpu, &mut bus, decode(0xB400_0040).unwrap()).unwrap();
    assert_eq!(cpu.regs.pc, 0x4000_0008);
}

#[test]
fn cbz_falls_through_when_nonzero() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.pc = 0x4000_0000;
    cpu.regs.set_x(0, 1);
    execute(&mut cpu, &mut bus, decode(0xB400_0040).unwrap()).unwrap();
    assert_eq!(cpu.regs.pc, 0x4000_0004);
}

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
fn str_wzr_sp_60() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.sp = 0x4000_0000;
    execute(&mut cpu, &mut bus, decode(0xB900_3FFF).unwrap()).unwrap();
    assert_eq!(bus.mem.read(0x4000_003C, 4), Some(0));
}

#[test]
fn ldr_str_roundtrip() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.set_x(1, 0x4000_0000);
    cpu.regs.set_x(0, 0xCAFE_0000_DEAD_BEEF);
    execute(&mut cpu, &mut bus, decode(0xF900_0020).unwrap()).unwrap();
    assert_eq!(bus.mem.read(0x4000_0000, 8), Some(0xCAFE_0000_DEAD_BEEF));
    execute(&mut cpu, &mut bus, decode(0xF940_0022).unwrap()).unwrap();
    assert_eq!(cpu.regs.x(2), 0xCAFE_0000_DEAD_BEEF);
}
