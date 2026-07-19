use super::*;

const NOP: u64 = 0xd503_201f;
const SEV: u64 = 0xd503_209f;
const SEVL: u64 = 0xd503_20bf;
const WFE: u64 = 0xd503_205f;

fn one_core_program(words: &[u64]) -> Machine {
    let mut machine = Machine::new(1);
    let code = RAM_BASE + 0x70_000;
    for (index, word) in words.iter().enumerate() {
        machine
            .bus
            .mem
            .write(code + index as u64 * INSTRUCTION_SIZE, 4, *word)
            .unwrap();
    }
    machine.cpus[0].regs.pc = code;
    machine
}

#[test]
fn wfe_parks_without_an_event() {
    let mut machine = one_core_program(&[WFE, NOP]);

    assert_eq!(machine.run(1), 1);
    assert_eq!(machine.cpus[0].lifecycle, CpuLifecycle::WaitingForInterrupt);
    assert!(machine.cpus[0].waiting_for_event);
    assert_eq!(machine.cooperative_wfe_parks, 1);
    assert_eq!(machine.run(1), 0);
}

#[test]
fn sevl_event_is_consumed_by_the_next_wfe() {
    let mut machine = one_core_program(&[SEVL, WFE, NOP]);

    assert_eq!(machine.run(2), 2);
    assert_eq!(machine.cpus[0].lifecycle, CpuLifecycle::Runnable);
    assert!(!machine.cpus[0].event_register);
    assert!(!machine.cpus[0].waiting_for_event);
    assert_eq!(machine.cooperative_wfe_parks, 0);
}

#[test]
fn sev_wakes_wfe_but_not_wfi_and_consumes_the_waking_event() {
    let mut machine = Machine::new(3);
    let code = RAM_BASE + 0x71_000;
    machine.bus.mem.write(code, 4, SEV).unwrap();
    machine.cpus[0].regs.pc = code;
    machine.cpus[1].lifecycle = CpuLifecycle::WaitingForInterrupt;
    machine.cpus[1].waiting_for_event = true;
    machine.cpus[2].lifecycle = CpuLifecycle::WaitingForInterrupt;

    assert_eq!(machine.run(1), 1);
    assert_eq!(machine.cpus[1].lifecycle, CpuLifecycle::Runnable);
    assert!(!machine.cpus[1].event_register);
    assert!(!machine.cpus[1].waiting_for_event);
    assert_eq!(machine.cpus[2].lifecycle, CpuLifecycle::WaitingForInterrupt);
    assert!(machine.cpus[2].event_register);
    assert!(!machine.cpus[2].waiting_for_event);
}

#[test]
fn timer_wakes_wfe_after_idle_time_fast_forward() {
    let mut machine = one_core_program(&[WFE, NOP]);
    machine.cpus[0].sys.vbar_el1 = RAM_BASE + 0x72_000;
    machine.cpus[0].sys.cntv_ctl_el0 = TIMER_CTL_ENABLE;
    machine.cpus[0].sys.cntv_cval_el0 = 100;

    assert_eq!(machine.run(1), 1);
    assert!(machine.cpus[0].waiting_for_event);
    assert_eq!(machine.run(1), 1);
    assert!(!machine.cpus[0].waiting_for_event);
    assert!(machine.virtual_time >= 101);
    assert_eq!(machine.cooperative_idle_fast_forward_cycles, 99);
}

#[test]
fn pending_private_irq_wakes_wfe() {
    let mut machine = one_core_program(&[WFE, NOP]);
    assert_eq!(machine.run(1), 1);
    assert!(machine.cpus[0].waiting_for_event);

    machine.bus.gic.enable_interrupt_for_cpu(0, 7);
    machine.bus.gic.set_pending_for_cpu(0, 7);

    assert_eq!(machine.run(1), 1);
    assert_eq!(machine.cpus[0].lifecycle, CpuLifecycle::Runnable);
    assert!(!machine.cpus[0].waiting_for_event);
}

#[cfg(not(any(target_arch = "wasm32", target_arch = "wasm64")))]
#[test]
fn native_parallel_sev_consumes_wfe_event_and_preserves_wfi_event() {
    use super::parallel_native::test_shared_run;
    use crate::runtime::parallel_native::{
        LIFE_RUNNABLE, LIFE_WAITING, LIFE_WAITING_EVENT, events,
    };
    use std::sync::atomic::Ordering;

    let mut bus = SystemBus::with_cpu_count(3);
    let shared = test_shared_run(&mut bus, &[LIFE_RUNNABLE, LIFE_RUNNABLE, LIFE_RUNNABLE]);
    let mut sender = Armv8Cpu::with_core(0);
    let mut wfe = Armv8Cpu::with_core(1);
    let mut wfi = Armv8Cpu::with_core(2);
    events::park_after_wait(1, &mut wfe, &shared, true);
    events::park_after_wait(2, &mut wfi, &shared, false);

    assert_eq!(
        shared.lifecycle[1].load(Ordering::Acquire),
        LIFE_WAITING_EVENT
    );
    assert_eq!(shared.lifecycle[2].load(Ordering::Acquire), LIFE_WAITING);
    events::broadcast(0, &mut sender, &shared);

    assert_eq!(shared.lifecycle[1].load(Ordering::Acquire), LIFE_RUNNABLE);
    assert!(!shared.event_registers[1].load(Ordering::Acquire));
    assert_eq!(shared.lifecycle[2].load(Ordering::Acquire), LIFE_WAITING);
    assert!(shared.event_registers[2].load(Ordering::Acquire));
}
