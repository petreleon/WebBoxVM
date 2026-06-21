use super::*;

#[test]
fn msr_daif_restores_all_daif_mask_bits() {
    let (mut cpu, mut bus) = setup();
    cpu.pstate = cpu.pstate.with_el(1).with_irq_masked(false);
    cpu.regs.set_x(0, PSTATE_DAIF_MASK);

    execute(
        &mut cpu,
        &mut bus,
        Instr {
            op: Opcode::Msr,
            rd: 0,
            rn: 0,
            rm: 0,
            imm: SYSREG_DAIF as u64,
            sf: true,
            cond: 0,
            size: 0,
        },
    )
    .unwrap();

    assert_eq!(cpu.pstate.daif(), PSTATE_DAIF_MASK);
    assert!(cpu.pstate.irq_masked());
}

#[test]
fn svc_from_el0_sets_syndrome_and_banks_stack() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.pc = 0x4000_0000;
    cpu.regs.sp = 0x7000_0000;
    cpu.sys.sp_el1 = 0x8000_0000;
    cpu.sys.vbar_el1 = 0xffff_8000_8000_0000;
    cpu.pstate = cpu.pstate.with_el(0).with_irq_masked(false);

    execute(&mut cpu, &mut bus, decode(0xD400_2461).unwrap()).unwrap(); // svc #0x123

    assert_eq!(cpu.regs.pc, cpu.sys.vbar_el1 + VBAR_SYNC_LOWER_EL_AARCH64);
    assert_eq!(cpu.sys.elr_el1, 0x4000_0004);
    assert_eq!(cpu.sys.spsr_el1 & PSTATE_EL_MASK, 0);
    assert_eq!(cpu.sys.esr_el1 >> 26, ESR_EC_SVC64);
    assert_eq!(cpu.sys.esr_el1 & 0xffff, 0x123);
    assert_eq!(cpu.sys.sp_el0, 0x7000_0000);
    assert_eq!(cpu.regs.sp, 0x8000_0000);
    assert_eq!(cpu.pstate.el(), 1);
    assert_eq!(cpu.pstate.daif(), PSTATE_DAIF_MASK);
}

#[test]
fn udf_from_el0_reports_unknown_reason_exception() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.pc = 0x4000_1000;
    cpu.regs.sp = 0x7000_0000;
    cpu.sys.sp_el1 = 0x8000_0000;
    cpu.sys.vbar_el1 = 0xffff_8000_8000_0000;
    cpu.pstate = cpu.pstate.with_el(0).with_irq_masked(false);

    execute(&mut cpu, &mut bus, decode(0x0000_1234).unwrap()).unwrap();

    assert_eq!(cpu.regs.pc, cpu.sys.vbar_el1 + VBAR_SYNC_LOWER_EL_AARCH64);
    assert_eq!(cpu.sys.elr_el1, 0x4000_1000);
    assert_eq!(cpu.sys.spsr_el1 & PSTATE_EL_MASK, 0);
    assert_eq!(cpu.sys.esr_el1, (ESR_EC_UNKNOWN << 26) | ESR_IL);
    assert_eq!(cpu.pstate.el(), 1);
}

#[test]
fn eret_to_el0_restores_user_stack_and_saves_kernel_stack() {
    let (mut cpu, mut bus) = setup();
    cpu.regs.pc = 0xffff_8000_8000_0400;
    cpu.regs.sp = 0xffff_0000_0000_8000;
    cpu.sys.sp_el0 = 0x0000_ffff_ff00_7000;
    cpu.sys.elr_el1 = 0x0000_aaaa_bbbb_c000;
    cpu.sys.spsr_el1 = cpu.pstate.with_el(0).with_irq_masked(false).to_u64();
    cpu.pstate = cpu.pstate.with_el(1).with_irq_masked(true);

    execute(&mut cpu, &mut bus, decode(0xD69F_03E0).unwrap()).unwrap();

    assert_eq!(cpu.regs.pc, 0x0000_aaaa_bbbb_c000);
    assert_eq!(cpu.pstate.el(), 0);
    assert_eq!(cpu.regs.sp, 0x0000_ffff_ff00_7000);
    assert_eq!(cpu.sys.sp_el1, 0xffff_0000_0000_8000);
}

#[test]
fn eret_clears_exclusive_monitor() {
    let (mut cpu, mut bus) = setup();
    cpu.sys.elr_el1 = 0x4000_1000;
    cpu.sys.spsr_el1 = cpu.pstate.with_el(1).to_u64();
    cpu.reserve_exclusive(RAM_BASE + 0x2000, 8);

    execute(&mut cpu, &mut bus, decode(0xD69F_03E0).unwrap()).unwrap();

    assert!(cpu.exclusive.is_none());
}
