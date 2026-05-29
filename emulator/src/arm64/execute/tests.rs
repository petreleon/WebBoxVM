use super::*;
use crate::arm64::decode::decode;
use crate::constants::{TIMER_CTL_ENABLE, VBAR_IRQ_CURRENT_EL};

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

#[test]
fn ccmp_immediate_compares_literal() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.set_w(5, 0);
    cpu.pstate.set_nzcv(false, false, false, false); // GE is true
    execute(&mut cpu, &mut bus, decode(0x7A40_A8A0).unwrap()).unwrap();
    assert!(cpu.pstate.z());
}

#[test]
fn ccmn_immediate_adds_literal() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.set_w(11, 0xffff_fff8);
    cpu.pstate.set_nzcv(false, true, false, false); // EQ is true
    execute(&mut cpu, &mut bus, decode(0x3A48_0960).unwrap()).unwrap();
    assert!(cpu.pstate.z());
    assert!(cpu.pstate.c());
}

#[test]
fn ldrsw_sign_extends_to_x_register() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.sp = 0x4000_0000;
    bus.mem.write(0x4000_0024, 4, 0xffff_fffc);
    execute(&mut cpu, &mut bus, decode(0xB980_27F9).unwrap()).unwrap();
    assert_eq!(cpu.regs.x(25), 0xffff_ffff_ffff_fffc);
}

#[test]
fn timer_irq_uses_current_el_spx_vector() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.pc = 0x4000_0000;
    cpu.sys.vbar_el1 = 0xffff_8000_8000_0000;
    cpu.sys.cycle_count = 10_001;
    cpu.sys.cntp_ctl_el0 = TIMER_CTL_ENABLE;
    cpu.sys.cntp_cval_el0 = 10_001;
    cpu.sys.cntp_tval_el0 = 0;
    cpu.pstate = cpu.pstate.with_el(1).with_irq_masked(false);

    execute(&mut cpu, &mut bus, decode(0xD503_201F).unwrap()).unwrap();

    assert_eq!(cpu.regs.pc, cpu.sys.vbar_el1 + VBAR_IRQ_CURRENT_EL);
    assert!(cpu.sys.irq_pending);
}

#[test]
fn disabled_timer_does_not_deliver_irq() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.pc = 0x4000_0000;
    cpu.sys.vbar_el1 = 0xffff_8000_8000_0000;
    cpu.sys.cycle_count = 10_001;
    cpu.sys.cntp_cval_el0 = 10_001;
    cpu.pstate = cpu.pstate.with_el(1).with_irq_masked(false);

    execute(&mut cpu, &mut bus, decode(0xD503_201F).unwrap()).unwrap();

    assert_eq!(cpu.regs.pc, 0x4000_0004);
    assert!(!cpu.sys.irq_pending);
}

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
fn ldxp_stlxp_pair_roundtrip() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.set_x(2, 0x4000_0000);
    cpu.regs.set_x(0, 0xAAAA);
    cpu.regs.set_x(1, 0xBBBB);

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
