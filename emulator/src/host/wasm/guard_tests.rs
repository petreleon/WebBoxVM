use super::parallel_api::{cancel_parallel_run, finish_parallel_run};
use super::*;
use crate::constants::PL011_UART_IRQ_ID;
use std::panic::{AssertUnwindSafe, catch_unwind};

#[test]
fn active_parallel_run_rejects_machine_exports_before_access() {
    let mut emulator = Emulator::new(Some(1));
    emulator.boot = Some(Box::new(BootContext {
        machine: Machine::new(1),
        dtb_addr: 0,
    }));
    let token = emulator.parallel_begin_kernel(10).unwrap();
    let pc = emulator.boot.as_ref().unwrap().machine.cpus[0].regs.pc;
    let steps = emulator.boot.as_ref().unwrap().machine.total_steps;

    assert_rejected(|| {
        let _ = emulator.pc();
    });
    assert_rejected(|| emulator.send_uart_bytes(vec![b'x']));
    assert_rejected(|| {
        let _ = emulator.run_kernel(1);
    });
    assert_rejected(|| {
        let _ = emulator.debug_read_pa_u64(0);
    });
    assert_rejected(|| {
        let _ = emulator.jit_compile_current_block(Some(0));
    });
    assert_rejected(|| emulator.jit_store_guest(Some(0), 0, 8, 1));
    assert_rejected(|| {
        let _ = emulator.parallel_begin_kernel(1);
    });
    assert_rejected(|| {
        let _ = emulator.parallel_worker_threads();
    });
    assert_rejected(|| {
        let _ = emulator.parallel_max_local_in_flight();
    });

    let machine = &emulator.boot.as_ref().unwrap().machine;
    assert_eq!(machine.cpus[0].regs.pc, pc);
    assert_eq!(machine.total_steps, steps);
    assert!(!machine.bus.gic.is_pending(PL011_UART_IRQ_ID));
    assert!(emulator.jit_pending_stores.is_empty());
    assert!(!emulator.jit_helper_failed);

    cancel_parallel_run(token).unwrap();
    assert!(finish_parallel_run(token).unwrap().starts_with("KERNEL:"));
    assert_eq!(emulator.pc(), pc);
    assert_eq!(emulator.parallel_worker_threads(), 0);
    assert_eq!(emulator.parallel_max_local_in_flight(), 0);
}

#[test]
fn access_lease_and_parallel_start_are_mutually_exclusive() {
    let mut emulator = Emulator::new(Some(1));
    let access = emulator.try_parallel_idle().unwrap();
    assert_rejected(|| {
        let _ = emulator.parallel_begin_kernel(1);
    });
    drop(access);

    let start = emulator.require_parallel_start();
    assert_rejected(|| {
        let _ = emulator.pc();
    });
    drop(start);
    assert_eq!(emulator.pc(), 0);
}

fn assert_rejected(call: impl FnOnce()) {
    assert!(
        catch_unwind(AssertUnwindSafe(call)).is_err(),
        "active parallel access should synchronously reject"
    );
}
