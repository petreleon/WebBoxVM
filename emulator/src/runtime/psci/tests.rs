use super::*;

mod power;

const NOP: u64 = 0xd503_201f;
const HVC_ZERO: u64 = 0xd400_0002;

fn psci_call(machine: &mut Machine, function: u64, arg1: u64, arg2: u64, arg3: u64) {
    let caller = &mut machine.cpus[0];
    caller.pstate = crate::arch::arm64::ProcessorState::el1h_masked();
    caller.regs.set_x(0, function);
    caller.regs.set_x(1, arg1);
    caller.regs.set_x(2, arg2);
    caller.regs.set_x(3, arg3);
    assert!(machine.handle_psci_call(
        0,
        Instr {
            op: Opcode::Hvc,
            ..Instr::nop()
        },
        2,
    ));
}

#[test]
fn cpu_on_resets_secondary_into_el1h_with_context_and_identity() {
    let mut machine = Machine::new(2);
    let entry = RAM_BASE + 0x1000;
    machine.bus.mem.write(entry, 4, NOP);
    machine.cpus[1].regs.set_x(9, 0xdead);

    psci_call(&mut machine, PSCI_CPU_ON64, 1, entry, 0xcafe);

    let secondary = &machine.cpus[1];
    assert_eq!(machine.cpus[0].regs.x(0), 0);
    assert_eq!(secondary.lifecycle, CpuLifecycle::Runnable);
    assert_eq!(secondary.regs.pc, entry);
    assert_eq!(secondary.regs.x(0), 0xcafe);
    assert_eq!(secondary.regs.x(9), 0);
    assert_eq!(secondary.sys.mpidr_el1, 0x8000_0001);
    assert_eq!(
        secondary.pstate,
        crate::arch::arm64::ProcessorState::el1h_masked()
    );
}

#[test]
fn cpu_on_rejects_an_online_cpu_and_invalid_entry() {
    let mut machine = Machine::new(2);
    let entry = RAM_BASE + 0x2000;
    machine.bus.mem.write(entry, 4, NOP);

    psci_call(&mut machine, PSCI_CPU_ON64, 1, entry, 0);
    psci_call(&mut machine, PSCI_CPU_ON64, 1, entry, 0);
    assert_eq!(machine.cpus[0].regs.x(0), psci_result(PSCI_ALREADY_ON));

    machine.cpus[1].lifecycle = CpuLifecycle::PoweredOff;
    psci_call(&mut machine, PSCI_CPU_ON64, 1, entry + 1, 0);
    assert_eq!(
        machine.cpus[0].regs.x(0),
        psci_result(PSCI_INVALID_PARAMETERS)
    );
}

#[test]
fn decoded_hvc_starts_the_secondary_through_the_run_loop() {
    let mut machine = Machine::new(2);
    let caller_pc = RAM_BASE + 0x3000;
    let entry = RAM_BASE + 0x4000;
    machine.bus.mem.write(caller_pc, 4, HVC_ZERO);
    machine.bus.mem.write(entry, 4, NOP);
    machine.cpus[0].regs.pc = caller_pc;
    machine.cpus[0].pstate = crate::arch::arm64::ProcessorState::el1h_masked();
    machine.cpus[0].regs.set_x(0, PSCI_CPU_ON64);
    machine.cpus[0].regs.set_x(1, 1);
    machine.cpus[0].regs.set_x(2, entry);
    machine.cpus[0].regs.set_x(3, 0x1234);

    assert_eq!(machine.run(1), 1);
    assert_eq!(machine.cpus[0].regs.pc, caller_pc + INSTRUCTION_SIZE);
    assert_eq!(machine.cpus[1].regs.pc, entry);
    assert_eq!(machine.active_core, 1);
}

#[test]
fn affinity_info_reports_power_state() {
    let mut machine = Machine::new(2);
    psci_call(&mut machine, PSCI_AFFINITY_INFO64, 1, 0, 0);
    assert_eq!(machine.cpus[0].regs.x(0), PSCI_AFFINITY_OFF as u64);

    machine.cpus[1].lifecycle = CpuLifecycle::Runnable;
    psci_call(&mut machine, PSCI_AFFINITY_INFO64, 1, 0, 0);
    assert_eq!(machine.cpus[0].regs.x(0), PSCI_AFFINITY_ON as u64);
}

#[test]
fn nonzero_hvc_immediate_is_not_a_psci_call() {
    let mut machine = Machine::new(1);
    machine.cpus[0].pstate = crate::arch::arm64::ProcessorState::el1h_masked();
    machine.cpus[0].regs.set_x(0, PSCI_CPU_OFF);

    assert!(!machine.handle_psci_call(
        0,
        Instr {
            op: Opcode::Hvc,
            imm: 1,
            ..Instr::nop()
        },
        1,
    ));
    assert_eq!(machine.cpus[0].lifecycle, CpuLifecycle::Runnable);
}

#[test]
fn el0_hvc_cannot_power_off_the_cpu() {
    let mut machine = Machine::new(1);
    let caller_pc = RAM_BASE + 0x5000;
    machine.bus.mem.write(caller_pc, 4, HVC_ZERO);
    machine.cpus[0].regs.pc = caller_pc;
    machine.cpus[0].pstate = crate::arch::arm64::ProcessorState::new()
        .with_el(0)
        .with_sp_select(false);
    machine.cpus[0].regs.set_x(0, PSCI_CPU_OFF);

    assert_eq!(machine.run(1), 1);
    assert_eq!(machine.cpus[0].lifecycle, CpuLifecycle::Runnable);
    assert_eq!(machine.cpus[0].regs.pc, VBAR_SYNC_LOWER_EL_AARCH64);
    assert_eq!(machine.cpus[0].sys.elr_el1, caller_pc);
    assert_eq!(machine.cpus[0].sys.esr_el1 >> 26, ESR_EC_UNKNOWN);
}
