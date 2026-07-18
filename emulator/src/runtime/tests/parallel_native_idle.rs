use super::parallel_native::test_shared_run;
use super::*;
use crate::runtime::parallel_native::{LIFE_RUNNABLE, LIFE_WAITING, idle};
use std::sync::atomic::Ordering;
use std::sync::mpsc::{self, RecvTimeoutError};
use std::time::{Duration, Instant};

#[test]
fn pending_wake_excludes_idle_fast_forward_snapshot() {
    let mut bus = SystemBus::with_cpu_count(2);
    bus.gic.enable_interrupt_for_cpu(0, 7);
    bus.gic.set_pending_for_cpu(0, 7);
    let shared = test_shared_run(&mut bus, &[LIFE_WAITING, LIFE_WAITING]);
    shared.deadlines[0].store(10, Ordering::Relaxed);
    shared.deadlines[1].store(1_000, Ordering::Relaxed);

    let bus_guard = shared
        .bus
        .write()
        .unwrap_or_else(|poison| poison.into_inner());
    let (gate_seen, coordinate_blocked, wake_result, coordinate_result) =
        std::thread::scope(|scope| {
            let (wake_tx, wake_rx) = mpsc::channel();
            let wake_shared = &shared;
            scope.spawn(move || {
                let mut cpu = Armv8Cpu::with_core(0);
                cpu.lifecycle = CpuLifecycle::WaitingForInterrupt;
                wake_tx.send(idle::wake_if_ready(0, &mut cpu, wake_shared))
            });

            let wait_started = Instant::now();
            let gate_seen = loop {
                if shared.idle_gate.try_read().is_err() {
                    break true;
                }
                if wait_started.elapsed() >= Duration::from_secs(1) {
                    break false;
                }
                std::thread::yield_now();
            };

            let (coordinate_tx, coordinate_rx) = mpsc::channel();
            let coordinate_shared = &shared;
            scope.spawn(move || {
                let mut cpu = Armv8Cpu::with_core(1);
                cpu.lifecycle = CpuLifecycle::WaitingForInterrupt;
                coordinate_tx.send(idle::coordinate(1, &cpu, coordinate_shared))
            });
            let early_coordinate = coordinate_rx.recv_timeout(Duration::from_millis(25));
            let coordinate_blocked = matches!(early_coordinate, Err(RecvTimeoutError::Timeout));
            drop(bus_guard);
            let coordinate_result = match early_coordinate {
                Ok(result) => Some(result),
                Err(RecvTimeoutError::Timeout) => {
                    coordinate_rx.recv_timeout(Duration::from_secs(1)).ok()
                }
                Err(RecvTimeoutError::Disconnected) => None,
            };
            (
                gate_seen,
                coordinate_blocked,
                wake_rx.recv_timeout(Duration::from_secs(1)).ok(),
                coordinate_result,
            )
        });

    assert!(gate_seen, "wake worker never acquired the transition gate");
    assert!(
        coordinate_blocked,
        "idle snapshot overlapped a pending wake"
    );
    assert_eq!(wake_result, Some(true));
    assert_eq!(coordinate_result, Some(false));
    assert_eq!(shared.lifecycle[0].load(Ordering::Acquire), LIFE_RUNNABLE);
    assert_eq!(shared.next_cycle.load(Ordering::Acquire), 0);
}

#[test]
fn coordinator_defers_to_pending_shared_irq_before_owner_wakes() {
    let mut bus = SystemBus::with_cpu_count(2);
    bus.gic.enable_interrupt_for_cpu(0, 7);
    bus.gic.set_pending_for_cpu(0, 7);
    let shared = test_shared_run(&mut bus, &[LIFE_WAITING, LIFE_WAITING]);
    shared.deadlines[0].store(100, Ordering::Relaxed);
    let mut coordinator = Armv8Cpu::with_core(1);
    coordinator.lifecycle = CpuLifecycle::WaitingForInterrupt;

    assert!(!idle::coordinate(1, &coordinator, &shared));
    assert!(!shared.stop.load(Ordering::Acquire));
    assert_eq!(shared.next_cycle.load(Ordering::Acquire), 0);

    let mut owner = Armv8Cpu::with_core(0);
    owner.lifecycle = CpuLifecycle::WaitingForInterrupt;
    assert!(idle::wake_if_ready(0, &mut owner, &shared));
    assert_eq!(shared.lifecycle[0].load(Ordering::Acquire), LIFE_RUNNABLE);
}
