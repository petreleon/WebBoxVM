use super::*;

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
    assert_eq!(cpu.sys.last_irq_id, PHYSICAL_TIMER_IRQ_ID);
}

#[test]
fn timer_irq_from_el0_uses_lower_vector_and_banks_stack() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.pc = 0x4000_0000;
    cpu.regs.sp = 0x7000_0000;
    cpu.sys.sp_el1 = 0x8000_0000;
    cpu.sys.vbar_el1 = 0xffff_8000_8000_0000;
    cpu.sys.cycle_count = 10_001;
    cpu.sys.cntp_ctl_el0 = TIMER_CTL_ENABLE;
    cpu.sys.cntp_cval_el0 = 10_001;
    cpu.pstate = cpu.pstate.with_el(0).with_irq_masked(false);

    execute(&mut cpu, &mut bus, decode(0xD503_201F).unwrap()).unwrap();

    assert_eq!(cpu.regs.pc, cpu.sys.vbar_el1 + VBAR_IRQ_LOWER_EL_AARCH64);
    assert_eq!(cpu.sys.spsr_el1 & PSTATE_EL_MASK, 0);
    assert_eq!(cpu.sys.sp_el0, 0x7000_0000);
    assert_eq!(cpu.regs.sp, 0x8000_0000);
    assert!(cpu.sys.irq_pending);
    assert_eq!(cpu.sys.last_irq_id, PHYSICAL_TIMER_IRQ_ID);
}

#[test]
fn virtual_timer_irq_uses_virtual_ppi() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.pc = 0x4000_0000;
    cpu.sys.vbar_el1 = 0xffff_8000_8000_0000;
    cpu.sys.cycle_count = 10_001;
    cpu.sys.cntv_ctl_el0 = TIMER_CTL_ENABLE;
    cpu.sys.cntv_cval_el0 = 10_001;
    cpu.pstate = cpu.pstate.with_el(1).with_irq_masked(false);

    execute(&mut cpu, &mut bus, decode(0xD503_201F).unwrap()).unwrap();

    assert_eq!(cpu.regs.pc, cpu.sys.vbar_el1 + VBAR_IRQ_CURRENT_EL);
    assert!(cpu.sys.irq_pending);
    assert_eq!(cpu.sys.last_irq_id, VIRTUAL_TIMER_IRQ_ID);
}

#[test]
fn virtual_timer_sysregs_track_tval_ctl_and_cntkctl() {
    let (mut cpu, _) = setup();
    cpu.sys.cycle_count = 100;
    cpu.sys.write_sys_reg(SYSREG_CNTV_TVAL_EL0, 25);
    cpu.sys.write_sys_reg(SYSREG_CNTV_CTL_EL0, TIMER_CTL_ENABLE);
    cpu.sys.write_sys_reg(SYSREG_CNTKCTL_EL1, 0x1234);

    assert_eq!(cpu.sys.cntv_cval_el0, 125);
    assert_eq!(
        cpu.sys.read_sys_reg(SYSREG_CNTV_CTL_EL0, 1),
        TIMER_CTL_ENABLE
    );
    assert_eq!(cpu.sys.read_sys_reg(SYSREG_CNTKCTL_EL1, 1), 0x1234);
}

#[test]
fn timer_tval_reads_count_down_and_accept_signed_deadlines() {
    let (mut cpu, _) = setup();
    cpu.sys.cycle_count = 100;

    cpu.sys.write_sys_reg(SYSREG_CNTP_TVAL_EL0, 25);
    cpu.sys.cycle_count = 110;
    assert_eq!(cpu.sys.read_sys_reg(SYSREG_CNTP_TVAL_EL0, 1), 15);

    cpu.sys.write_sys_reg(SYSREG_CNTV_TVAL_EL0, u32::MAX as u64);
    assert_eq!(cpu.sys.cntv_cval_el0, 109);
    assert_eq!(
        cpu.sys.read_sys_reg(SYSREG_CNTV_TVAL_EL0, 1),
        u32::MAX as u64
    );
}

#[test]
fn masked_timers_do_not_wake_wfi_deadline() {
    let (mut cpu, _) = setup();
    cpu.sys.cycle_count = 100;
    cpu.sys.cntv_cval_el0 = 125;
    cpu.sys.cntv_ctl_el0 = TIMER_CTL_ENABLE | TIMER_CTL_IMASK;

    assert_eq!(cpu.sys.next_timer_deadline(), None);
}

#[test]
fn timer_irq_check_needed_tracks_pending_and_unmasked_timers() {
    let (mut cpu, _) = setup();

    assert!(!cpu.sys.timer_irq_check_needed());
    cpu.sys.vbar_el1 = 0x8000;
    assert!(!cpu.sys.timer_irq_check_needed());

    cpu.sys.cntv_ctl_el0 = TIMER_CTL_ENABLE | TIMER_CTL_IMASK;
    assert!(!cpu.sys.timer_irq_check_needed());

    cpu.sys.cntv_cval_el0 = 10;
    cpu.sys.cntv_ctl_el0 = TIMER_CTL_ENABLE;
    assert!(!cpu.sys.timer_irq_check_needed());

    cpu.sys.cycle_count = 10;
    assert!(cpu.sys.timer_irq_check_needed());

    cpu.sys.cntv_ctl_el0 = 0;
    cpu.sys.irq_pending = true;
    assert!(cpu.sys.timer_irq_check_needed());
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
