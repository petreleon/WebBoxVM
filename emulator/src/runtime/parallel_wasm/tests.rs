use super::*;
use std::sync::atomic::Ordering;

mod exclusive_event;
mod idle_polling;

fn begin_parallel(machine: &mut Machine, max_steps: usize) -> u64 {
    let access = WasmAccessControl::new();
    let start = access.try_parallel_start().unwrap();
    machine.begin_parallel_wasm(max_steps, start).unwrap()
}

#[test]
fn forged_and_expired_tokens_are_rejected() {
    assert!(registry::claim(0xdead_beef, 0).is_err());
    let mut machine = Machine::new(1);
    let token = begin_parallel(&mut machine, 1);
    Machine::cancel_parallel_wasm(token).unwrap();
    Machine::finish_parallel_wasm(token).unwrap();
    assert!(registry::claim(token, 0).is_err());
}

#[test]
fn core_claim_is_unique_and_finish_waits_for_lease_drop() {
    let mut machine = Machine::new(2);
    let token = begin_parallel(&mut machine, 10);
    let lease = registry::claim(token, 0).unwrap();
    assert!(registry::claim(token, 0).is_err());
    assert!(Machine::finish_parallel_wasm(token).is_err());
    drop(lease);
    Machine::cancel_parallel_wasm(token).unwrap();
    Machine::finish_parallel_wasm(token).unwrap();
}

#[test]
fn finish_claim_keeps_idle_guards_closed_until_finalize_completes() {
    let mut machine = Machine::new(1);
    let token = begin_parallel(&mut machine, 1);
    Machine::cancel_parallel_wasm(token).unwrap();
    let claim = registry::close(token).unwrap();
    assert!(machine.parallel_wasm_active());
    assert!(registry::close(token).is_err());
    registry::complete_finalize(claim);
    assert!(!machine.parallel_wasm_active());
}

#[test]
fn worker_completion_raii_allows_join_after_every_core_returns() {
    let mut machine = Machine::new(2);
    let token = begin_parallel(&mut machine, 100);
    std::thread::scope(|scope| {
        for core in 0..2 {
            scope.spawn(move || Machine::run_parallel_wasm_core(token, core).unwrap());
        }
    });
    Machine::finish_parallel_wasm(token).unwrap();
    assert_eq!(machine.parallel_run_stats().worker_threads, 2);
}

#[test]
fn begin_publishes_a_waiting_cpu_timer_deadline() {
    let mut machine = Machine::new(1);
    machine.cpus[0].lifecycle = CpuLifecycle::WaitingForInterrupt;
    machine.cpus[0].sys.cycle_count = 40;
    machine.cpus[0].sys.cntp_ctl_el0 = TIMER_CTL_ENABLE;
    machine.cpus[0].sys.cntp_cval_el0 = 75;
    let token = begin_parallel(&mut machine, 10);
    assert_eq!(
        machine.wasm_parallel.deadlines[0].load(Ordering::Acquire),
        75
    );
    Machine::cancel_parallel_wasm(token).unwrap();
    Machine::finish_parallel_wasm(token).unwrap();
}

#[test]
fn coordinator_defers_to_waiting_core_with_refreshed_gic_interrupt() {
    let mut machine = Machine::new(2);
    for cpu in &mut machine.cpus {
        cpu.lifecycle = CpuLifecycle::WaitingForInterrupt;
    }
    machine.bus.gic.enable_interrupt(PL011_UART_IRQ_ID);
    let target = machine.bus.gic.cpu_affinity(1).unwrap();
    machine
        .bus
        .gic
        .set_interrupt_route(PL011_UART_IRQ_ID, target);
    machine.bus.write(UART_BASE + UART_IMSC_OFFSET, 4, 0x50);
    machine.bus.feed_uart_input("x");
    machine.bus.clear_irq_pending(PL011_UART_IRQ_ID);
    assert!(!machine.bus.gic.has_pending_enabled_for_cpu(1));

    let token = begin_parallel(&mut machine, 10);
    assert!(!idle::coordinate(
        0,
        &machine.cpus[0],
        &machine.wasm_parallel
    ));
    assert!(!machine.wasm_parallel.stop.load(Ordering::Acquire));
    assert!(machine.bus.gic.has_pending_enabled_for_cpu(1));
    Machine::cancel_parallel_wasm(token).unwrap();
    Machine::finish_parallel_wasm(token).unwrap();
}

#[test]
fn system_off_latch_prevents_boot_ready_resurrection() {
    let mut machine = Machine::new(2);
    let token = begin_parallel(&mut machine, 10);
    machine.wasm_parallel.lifecycle[1].store(LIFE_BOOT_READY, Ordering::Release);
    machine
        .wasm_parallel
        .system_off
        .store(true, Ordering::Release);
    machine
        .wasm_parallel
        .reset_requested
        .store(true, Ordering::Release);
    Machine::cancel_parallel_wasm(token).unwrap();
    Machine::finish_parallel_wasm(token).unwrap();
    assert!(
        machine
            .cpus
            .iter()
            .all(|cpu| cpu.lifecycle == CpuLifecycle::PoweredOff)
    );
}

#[test]
fn drop_claim_keeps_access_terminal_across_finalize() {
    let access = WasmAccessControl::new();
    access.try_parallel_start().unwrap().commit();
    assert!(matches!(access.claim_drop(), WasmDropAccess::Leak));
    access.finish_parallel();
    assert!(access.try_idle().is_err());
}
