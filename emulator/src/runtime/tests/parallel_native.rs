use super::*;
use crate::runtime::parallel_native::{
    LIFE_OFF, LIFE_RUNNABLE, NO_DEADLINE, SharedRun, idle, psci,
};
use crate::runtime::psci::{PSCI_CPU_ON64, PSCI_SYSTEM_OFF};
use std::sync::RwLock;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU64, AtomicUsize, Ordering};

const ADD_X0_X0_1: u64 = 0x9100_0400;
const BRANCH_BACK_ONE: u64 = 0x17ff_ffff;
const HVC_ZERO: u64 = 0xd400_0002;
const NOP: u64 = 0xd503_201f;

fn parallel_compute_machine() -> Machine {
    let mut machine = Machine::new(2);
    let code = RAM_BASE + 0x20_000;
    machine.bus.mem.write(code, 4, ADD_X0_X0_1).unwrap();
    machine
        .bus
        .mem
        .write(code + INSTRUCTION_SIZE, 4, BRANCH_BACK_ONE)
        .unwrap();
    for cpu in &mut machine.cpus {
        cpu.lifecycle = CpuLifecycle::Runnable;
        cpu.regs.pc = code;
    }
    machine.set_run_backend(RunBackend::NativeThreads);
    machine
}

#[test]
fn native_backend_uses_one_worker_per_vcpu_and_exact_budget() {
    let mut machine = parallel_compute_machine();

    assert_eq!(machine.run(200_000), 200_000);
    assert_eq!(machine.total_steps, 200_000);
    assert!(machine.cpus.iter().all(|cpu| cpu.regs.x(0) > 0));
    assert_eq!(machine.parallel_run_stats().worker_threads, 2);
}

#[test]
fn native_backend_observes_cpu_local_execution_in_parallel() {
    let mut machine = parallel_compute_machine();

    machine.run(1_000_000);

    assert!(
        machine.parallel_run_stats().max_local_in_flight >= 2,
        "two vCPU workers never overlapped outside the shared bus lock"
    );
}

#[test]
fn spawn_failure_finishes_budget_serially_and_downgrades() {
    let mut machine = parallel_compute_machine();

    let ran = crate::runtime::parallel_native::spawn::with_failure_after(1, || machine.run(50_000));

    assert_eq!(ran, 50_000);
    assert_eq!(machine.total_steps, 50_000);
    assert_eq!(machine.run_backend(), RunBackend::Serial);
    assert_eq!(machine.parallel_run_stats().worker_threads, 1);
    assert!(machine.cpus.iter().all(|cpu| cpu.regs.x(0) > 0));
}

#[test]
fn native_backend_initializes_waiting_timer_deadlines() {
    let mut machine = Machine::new(2);
    let code = RAM_BASE + 0x24_000;
    machine.bus.mem.write(code, 4, NOP).unwrap();
    machine.cpus[0].regs.pc = code;
    machine.cpus[0].lifecycle = CpuLifecycle::WaitingForInterrupt;
    machine.cpus[0].sys.vbar_el1 = RAM_BASE + 0x28_000;
    machine.cpus[0].sys.cntv_ctl_el0 = TIMER_CTL_ENABLE;
    machine.cpus[0].sys.cntv_cval_el0 = 100;
    machine.set_run_backend(RunBackend::NativeThreads);

    assert_eq!(machine.run(1), 1);
    assert!(machine.virtual_time >= 101);
    assert_eq!(machine.cpus[0].lifecycle, CpuLifecycle::Runnable);
}

#[test]
fn system_off_latch_rejects_late_cpu_on_and_wfi_publication() {
    let mut bus = SystemBus::with_cpu_count(3);
    let entry = RAM_BASE + 0x2c_000;
    bus.mem.write(entry, 4, NOP).unwrap();
    let shared = test_shared_run(&mut bus, &[LIFE_RUNNABLE, LIFE_RUNNABLE, LIFE_OFF]);
    let hvc = Instr {
        op: Opcode::Hvc,
        ..Instr::nop()
    };
    let mut shutdown_cpu = Armv8Cpu::with_core(0);
    shutdown_cpu.pstate = crate::arch::arm64::ProcessorState::el1h_masked();
    shutdown_cpu.regs.set_x(0, PSCI_SYSTEM_OFF);
    assert!(psci::handle(0, &mut shutdown_cpu, hvc, &shared));

    let mut late_cpu = Armv8Cpu::with_core(1);
    late_cpu.pstate = crate::arch::arm64::ProcessorState::el1h_masked();
    late_cpu.regs.set_x(0, PSCI_CPU_ON64);
    late_cpu.regs.set_x(1, 2);
    late_cpu.regs.set_x(2, entry);
    assert!(psci::handle(1, &mut late_cpu, hvc, &shared));
    idle::park_after_wfi(1, &mut late_cpu, &shared);

    assert!(shared.system_off.load(Ordering::Acquire));
    assert!(
        shared
            .lifecycle
            .iter()
            .all(|state| state.load(Ordering::Acquire) == LIFE_OFF)
    );
    assert_eq!(late_cpu.lifecycle, CpuLifecycle::PoweredOff);

    let mut machine = Machine::new(2);
    let shutdown = RAM_BASE + 0x30_000;
    machine.bus.mem.write(shutdown, 4, HVC_ZERO).unwrap();
    machine.cpus[0].regs.pc = shutdown;
    machine.cpus[0].regs.set_x(0, PSCI_SYSTEM_OFF);
    machine.cpus[0].pstate = crate::arch::arm64::ProcessorState::el1h_masked();
    machine.set_run_backend(RunBackend::NativeThreads);
    machine.run(10);
    assert!(
        machine
            .cpus
            .iter()
            .all(|cpu| cpu.lifecycle == CpuLifecycle::PoweredOff)
    );
}

pub(super) fn test_shared_run<'a>(bus: &'a mut SystemBus, states: &[u8]) -> SharedRun<'a> {
    let cores = states.len();
    SharedRun {
        bus: RwLock::new(bus),
        idle_gate: RwLock::new(()),
        remaining: AtomicUsize::new(10),
        next_cycle: AtomicU64::new(0),
        stop: AtomicBool::new(false),
        system_off: AtomicBool::new(false),
        reset_requested: AtomicBool::new(false),
        memory_epoch: AtomicU64::new(0),
        tlb_epoch: AtomicU64::new(0),
        lifecycle: states.iter().copied().map(AtomicU8::new).collect(),
        deadlines: (0..cores).map(|_| AtomicU64::new(NO_DEADLINE)).collect(),
        power_entry: (0..cores).map(|_| AtomicU64::new(0)).collect(),
        power_context: (0..cores).map(|_| AtomicU64::new(0)).collect(),
        fetch_faults: AtomicU64::new(0),
        exec_faults: AtomicU64::new(0),
        local_in_flight: AtomicUsize::new(0),
        max_local_in_flight: AtomicUsize::new(0),
    }
}
