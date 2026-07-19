use super::*;

const STR_X0_X1: u64 = 0xf900_0020;

#[test]
fn parallel_store_conservatively_wakes_wfe_but_not_wfi() {
    let mut machine = Machine::new(3);
    let code = RAM_BASE + 0x7a_000;
    let data = RAM_BASE + 0x7b_000;
    machine.bus.mem.write(code, 4, STR_X0_X1).unwrap();
    machine.cpus[0].regs.pc = code;
    machine.cpus[0].regs.set_x(0, 0xfeed_face);
    machine.cpus[0].regs.set_x(1, data);
    machine.cpus[0].reserve_exclusive(data, 8);

    machine.cpus[1].reserve_exclusive(data, 8);
    machine.cpus[1].lifecycle = CpuLifecycle::WaitingForInterrupt;
    machine.cpus[1].waiting_for_event = true;
    machine.cpus[2].reserve_exclusive(data, 8);
    machine.cpus[2].lifecycle = CpuLifecycle::WaitingForInterrupt;

    let token = begin_parallel(&mut machine, 1);
    std::thread::scope(|scope| {
        for core in 0..3 {
            scope.spawn(move || Machine::run_parallel_wasm_core(token, core).unwrap());
        }
    });
    assert_eq!(Machine::finish_parallel_wasm(token).unwrap().0, 1);

    assert!(!machine.cpus[0].event_register);
    assert_eq!(machine.cpus[1].lifecycle, CpuLifecycle::Runnable);
    assert!(!machine.cpus[1].waiting_for_event);
    assert!(!machine.cpus[1].event_register);
    assert_eq!(machine.cpus[2].lifecycle, CpuLifecycle::WaitingForInterrupt);
    assert!(!machine.cpus[2].waiting_for_event);
    assert!(machine.cpus[2].event_register);
}
