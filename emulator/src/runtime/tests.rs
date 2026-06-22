use super::*;

#[test]
fn load_store_translation_fault_enters_data_abort_vector() {
    let mut machine = Machine::new(1);
    let pc_va = KERNEL_VA_BASE;
    let pc_pa = 0x4000_0000;
    let data_va = 0x0000_AAAA_DAD7_03F8;

    machine.bus.mem.write(pc_pa, 4, 0xF900_001F);
    machine
        .bus
        .mem
        .write(0x1000 + 256 * 8, 8, (0x2000 & DESC_ADDR_MASK) | DESC_VALID);
    machine
        .bus
        .mem
        .write(0x2000, 8, (0x3000 & DESC_ADDR_MASK) | DESC_VALID);
    machine
        .bus
        .mem
        .write(0x3000, 8, (0x4000 & DESC_ADDR_MASK) | DESC_VALID);
    machine
        .bus
        .mem
        .write(0x4000, 8, (pc_pa & DESC_ADDR_MASK) | DESC_VALID);

    let cpu = machine.core_mut(0);
    cpu.regs.pc = pc_va;
    cpu.regs.set_x(0, data_va);
    cpu.pstate = cpu.pstate.with_el(1);
    cpu.sys.vbar_el1 = KERNEL_VA_BASE + 0x1000;
    cpu.sys.ttbr0_el1 = 0x5000;
    cpu.sys.ttbr1_el1 = 0x1000;
    cpu.sys.tcr_el1 = (16 << TCR_T1SZ_SHIFT) | 16;
    cpu.sys.sctlr_el1 = SCTLR_MMU_ENABLE;

    machine.run(1);

    let cpu = machine.core(0);
    assert_eq!(cpu.regs.pc, cpu.sys.vbar_el1 + VBAR_SYNC_CURRENT_EL);
    assert_eq!(cpu.sys.far_el1, data_va);
    assert_eq!(cpu.sys.elr_el1, pc_va);
    assert_eq!(cpu.sys.esr_el1 >> 26, ESR_EC_DATA_ABORT_CURRENT_EL);
    assert_ne!(cpu.sys.esr_el1 & ESR_DATA_ABORT_WNR, 0);
}

#[test]
fn fp_simd_trap_predicate_respects_el_and_fpen() {
    let mut cpu = Armv8Cpu::new();

    cpu.pstate = cpu.pstate.with_el(1);
    cpu.sys.cpacr_el1 = CPACR_FPEN_TRAP_NONE << CPACR_FPEN_SHIFT;
    assert!(!fp_simd_access_traps(&cpu));

    cpu.sys.cpacr_el1 = CPACR_FPEN_TRAP_EL0_EL1 << CPACR_FPEN_SHIFT;
    assert!(fp_simd_access_traps(&cpu));

    cpu.pstate = cpu.pstate.with_el(0);
    cpu.sys.cpacr_el1 = CPACR_FPEN_TRAP_EL1_EL0 << CPACR_FPEN_SHIFT;
    assert!(fp_simd_access_traps(&cpu));
}

#[test]
fn fp_simd_trap_fast_path_ignores_non_simd_instruction() {
    let mut machine = Machine::new(1);
    machine.core_mut(0).sys.cpacr_el1 = CPACR_FPEN_TRAP_EL0_EL1 << CPACR_FPEN_SHIFT;

    assert!(!machine.handle_fp_simd_trap(0, 0, 0, Instr::nop(), machine.trace.options, 1));
    assert_eq!(machine.total_steps, 0);
    assert_eq!(machine.active_core, 0);
}

#[test]
fn finish_core_preserves_single_core_round_robin_without_modulo() {
    let mut machine = Machine::new(1);

    machine.finish_core(0, 1);
    machine.finish_core(0, 1);

    assert_eq!(machine.total_steps, 2);
    assert_eq!(machine.active_core, 0);
}

#[test]
fn finish_core_wraps_multi_core_round_robin() {
    let mut machine = Machine::new(3);

    machine.finish_core(0, 3);
    assert_eq!(machine.active_core, 1);

    machine.finish_core(1, 3);
    assert_eq!(machine.active_core, 2);

    machine.finish_core(2, 3);
    assert_eq!(machine.total_steps, 3);
    assert_eq!(machine.active_core, 0);
}

#[test]
fn report_progress_keeps_threshold_before_due() {
    let mut machine = Machine::new(1);
    let mut next_report_at = 1_000_000;
    machine.total_steps = 999_999;

    machine.report_progress(0, &mut next_report_at, 0);

    assert_eq!(next_report_at, 1_000_000);
}

#[test]
fn report_progress_advances_threshold_when_due() {
    let mut machine = Machine::new(1);
    let mut next_report_at = 1_000_000;
    machine.total_steps = 1_000_000;

    machine.report_progress(0, &mut next_report_at, 0);

    assert_eq!(next_report_at, 2_000_000);
}

#[test]
fn report_progress_threshold_is_absolute_for_resumed_runs() {
    let mut machine = Machine::new(1);
    let start_steps = 100_000_000;
    let mut next_report_at = start_steps + 1_000_000;
    machine.total_steps = next_report_at;

    machine.report_progress(start_steps, &mut next_report_at, 0);

    assert_eq!(next_report_at, start_steps + 2_000_000);
}

#[test]
fn clear_exclusive_overlaps_updates_all_cores() {
    let mut machine = Machine::new(2);
    machine.core_mut(0).reserve_exclusive(RAM_BASE + 0x40, 8);
    machine.core_mut(1).reserve_exclusive(RAM_BASE + 0x80, 8);

    machine.clear_exclusive_overlaps(RAM_BASE + 0x42, 4);

    assert!(!machine.core(0).exclusive_matches(RAM_BASE + 0x40, 8));
    assert!(machine.core(1).exclusive_matches(RAM_BASE + 0x80, 8));
}

#[test]
fn gic_access_fast_path_ignores_non_sysreg_instruction() {
    let mut machine = Machine::new(1);

    assert!(!machine.handle_gic_access(0, Instr::nop(), 1));
    assert_eq!(machine.total_steps, 0);
    assert_eq!(machine.active_core, 0);
}
